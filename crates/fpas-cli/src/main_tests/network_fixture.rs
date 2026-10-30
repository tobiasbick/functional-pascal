//! Owned loopback fixtures with cancellation and absolute I/O deadlines.

use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const DEADLINE: Duration = Duration::from_secs(20);
const IO_TIMEOUT: Duration = Duration::from_secs(5);
const IO_SLICE: Duration = Duration::from_millis(50);

struct State {
    listener: TcpListener,
    cancelled: Arc<AtomicBool>,
    sockets: Mutex<Vec<Weak<TcpStream>>>,
    deadline: Instant,
}

pub(super) struct Listener {
    state: Arc<State>,
}

impl Listener {
    pub(super) fn bind(address: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            state: Arc::new(State {
                listener,
                cancelled: Arc::new(AtomicBool::new(false)),
                sockets: Mutex::new(Vec::new()),
                deadline: Instant::now() + DEADLINE,
            }),
        })
    }

    pub(super) fn local_addr(&self) -> io::Result<SocketAddr> {
        self.state.listener.local_addr()
    }

    pub(super) fn spawn<T: Send + 'static>(
        self,
        run: impl FnOnce(Self) -> T + Send + 'static,
    ) -> Server<T> {
        let state = Arc::clone(&self.state);
        Server {
            state,
            worker: Some(thread::spawn(move || run(self))),
        }
    }

    pub(super) fn accept(&self) -> io::Result<(Stream, SocketAddr)> {
        loop {
            match self.try_accept() {
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                }
                result => return result,
            }
        }
    }

    pub(super) fn try_accept(&self) -> io::Result<(Stream, SocketAddr)> {
        if self.state.cancelled.load(Ordering::Acquire) {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "fixture cancelled",
            ));
        }
        if Instant::now() >= self.state.deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "fixture accept deadline",
            ));
        }
        let (socket, address) = self.state.listener.accept()?;
        // Windows accepted sockets can inherit the listener's nonblocking mode.
        socket.set_nonblocking(false)?;
        socket.set_read_timeout(Some(IO_SLICE))?;
        socket.set_write_timeout(Some(IO_SLICE))?;
        let socket = Arc::new(socket);
        self.state
            .sockets
            .lock()
            .expect("fixture socket registry")
            .push(Arc::downgrade(&socket));
        if self.state.cancelled.load(Ordering::Acquire) {
            let _ = socket.shutdown(Shutdown::Both);
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "fixture cancelled",
            ));
        }
        Ok((
            Stream {
                socket,
                cancelled: Arc::clone(&self.state.cancelled),
                deadline: self.state.deadline,
            },
            address,
        ))
    }
}

pub(super) struct Stream {
    socket: Arc<TcpStream>,
    cancelled: Arc<AtomicBool>,
    deadline: Instant,
}

impl Stream {
    fn remaining(&self, deadline: Instant) -> io::Result<Duration> {
        if self.cancelled.load(Ordering::Acquire) {
            // Interrupted would make read_exact/write_all retry cancellation forever.
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "fixture cancelled",
            ));
        }
        deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .map(|remaining| remaining.min(IO_SLICE))
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "fixture I/O deadline"))
    }
}

impl Deref for Stream {
    type Target = TcpStream;

    fn deref(&self) -> &TcpStream {
        &self.socket
    }
}

impl Read for Stream {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let deadline = self.deadline.min(Instant::now() + IO_TIMEOUT);
        loop {
            self.socket
                .set_read_timeout(Some(self.remaining(deadline)?))?;
            match self.socket.as_ref().read(bytes) {
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) => {}
                result => return result,
            }
        }
    }
}

impl Write for Stream {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let deadline = self.deadline.min(Instant::now() + IO_TIMEOUT);
        loop {
            self.socket
                .set_write_timeout(Some(self.remaining(deadline)?))?;
            match self.socket.as_ref().write(bytes) {
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) => {}
                result => return result,
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.socket.as_ref().flush()
    }
}

pub(super) struct Server<T> {
    state: Arc<State>,
    worker: Option<JoinHandle<T>>,
}

impl<T> Server<T> {
    pub(super) fn join(mut self) -> thread::Result<T> {
        self.worker.take().expect("fixture worker").join()
    }
}

impl<T> Drop for Server<T> {
    fn drop(&mut self) {
        self.state.cancelled.store(true, Ordering::Release);
        let sockets = self
            .state
            .sockets
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        for socket in sockets.iter().filter_map(Weak::upgrade) {
            let _ = socket.shutdown(Shutdown::Both);
        }
        drop(sockets);
        if let Some(worker) = self.worker.take() {
            // An original client panic must survive a secondary fixture failure.
            let _ = worker.join();
        }
    }
}

#[test]
fn dropping_fixture_cancels_accept_and_releases_listener() {
    let listener = Listener::bind("127.0.0.1:0").expect("fixture listener");
    let address = listener.local_addr().expect("fixture address");
    let server = listener.spawn(|listener| {
        assert_eq!(
            listener.accept().err().expect("cancelled accept").kind(),
            io::ErrorKind::Interrupted
        );
    });
    drop(server);
    TcpListener::bind(address).expect("listener must have been released");
}

#[test]
fn dropping_fixture_interrupts_stalled_reader() {
    let listener = Listener::bind("127.0.0.1:0").expect("fixture listener");
    let address = listener.local_addr().expect("fixture address");
    let (ready, accepted) = std::sync::mpsc::channel();
    let server = listener.spawn(move |listener| {
        let (mut socket, _) = listener.accept().expect("fixture connection");
        ready.send(()).expect("accepted notification");
        let _ = socket.read_exact(&mut [0]);
    });
    let _peer = TcpStream::connect(address).expect("stalled peer");
    accepted
        .recv_timeout(IO_TIMEOUT)
        .expect("server accepted peer");
    let started = Instant::now();
    drop(server);
    assert!(
        started.elapsed() < IO_TIMEOUT,
        "cleanup waited for the socket read timeout"
    );
    TcpListener::bind(address).expect("listener must have been released");
}

#[test]
fn client_failure_survives_fixture_failure_during_cleanup() {
    let original = std::panic::catch_unwind(|| {
        let listener = Listener::bind("127.0.0.1:0").expect("fixture listener");
        let _server = listener.spawn(|listener| {
            listener.accept().expect("a client which never connects");
        });
        panic!("original client failure");
    })
    .expect_err("client failure must remain visible");
    assert_eq!(
        original.downcast_ref::<&str>(),
        Some(&"original client failure")
    );
}
