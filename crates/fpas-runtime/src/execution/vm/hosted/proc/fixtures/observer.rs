//! Host-owned sockets stay independent of VM teardown and prove child termination.

use std::io::{self, Read};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

pub struct Observer {
    listener: TcpListener,
    timeout: Duration,
}

impl Observer {
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(10))
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        Self { listener, timeout }
    }

    pub fn port(&self) -> String {
        self.listener.local_addr().unwrap().port().to_string()
    }

    pub fn assert_closed(self, count: usize) {
        let deadline = Instant::now() + self.timeout;
        for _ in 0..count {
            let stream = loop {
                assert!(
                    Instant::now() < deadline,
                    "child never connected to its host observer"
                );
                match self.listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => panic!("observer accept: {error}"),
                }
            };
            assert_stream_closed(stream, deadline);
        }
    }
}

pub(super) fn assert_stream_closed(mut stream: TcpStream, deadline: Instant) {
    // Windows accepted sockets can inherit the listener's nonblocking mode.
    stream.set_nonblocking(false).unwrap();
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .expect("child survived the host observer deadline");
        stream.set_read_timeout(Some(remaining)).unwrap();
        match stream.read(&mut [0]) {
            Ok(0) => return,
            Err(error) if error.kind() == io::ErrorKind::ConnectionReset => return,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Ok(_) => panic!("child sent data instead of closing its host observer"),
            Err(error) => panic!("child survived VM completion or abandonment: {error}"),
        }
    }
}
