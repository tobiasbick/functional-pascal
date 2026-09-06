//! VM-owned TCP listener registry.

use std::collections::HashMap;
use std::io;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::tls::server::TlsServer;
use super::transport::Transport;

const HANDLE_TAG: u64 = 0x4E4C_0000_0000_0000;
const HANDLE_TAG_MASK: u64 = 0xFFFF_0000_0000_0000;

/// Thread-safe TCP and TLS listeners owned by one VM.
pub(in crate::vm::hosted) struct NetworkListeners {
    next_handle: AtomicU64,
    listeners: Mutex<HashMap<u64, Arc<Listener>>>,
    shutdown: AtomicBool,
}

struct Listener {
    socket: TcpListener,
    mode: ListenerMode,
    closed: AtomicBool,
}

impl Listener {
    fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }

    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}

enum ListenerMode {
    Tcp,
    Tls(TlsServer),
}

impl NetworkListeners {
    /// Create an empty per-VM listener registry.
    pub(in crate::vm::hosted) fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(HANDLE_TAG | 1),
            listeners: Mutex::new(HashMap::new()),
            shutdown: AtomicBool::new(false),
        }
    }

    /// Bind a TCP listener and return its opaque runtime handle.
    pub(super) fn listen(&self, host: &str, port: i64) -> Result<u64, String> {
        self.bind(host, port, ListenerMode::Tcp)
    }

    /// Bind a TLS listener configured from PEM files.
    pub(super) fn listen_tls(
        &self,
        host: &str,
        port: i64,
        certificate_path: &str,
        private_key_path: &str,
        handshake_timeout_millis: i64,
    ) -> Result<u64, String> {
        let tls = TlsServer::from_pem_files(
            certificate_path,
            private_key_path,
            handshake_timeout_millis,
        )?;
        self.bind(host, port, ListenerMode::Tls(tls))
    }

    fn bind(&self, host: &str, port: i64, mode: ListenerMode) -> Result<u64, String> {
        let port = listener_port(port)?;
        let socket = TcpListener::bind((host, port)).map_err(|error| {
            format!("Could not bind network listener on {host}:{port}: {error}")
        })?;
        socket
            .set_nonblocking(true)
            .map_err(|error| format!("Could not configure network listener: {error}"))?;
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        let listener = Arc::new(Listener {
            socket,
            mode,
            closed: AtomicBool::new(false),
        });
        let mut listeners = self
            .listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if self.shutdown.load(Ordering::Acquire) {
            return Err("Network listener cannot be opened after VM shutdown".to_string());
        }
        listeners.insert(handle, listener);
        Ok(handle)
    }

    /// Accept one TCP or TLS connection from a listener.
    pub(super) fn accept(&self, handle: u64) -> Result<Transport, String> {
        self.accept_until_cancelled(handle, || false)
    }

    /// Accept one TCP or TLS connection until the supplied token is cancelled.
    pub(super) fn accept_with_cancellation(
        &self,
        handle: u64,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<Transport, String> {
        self.accept_until_cancelled(handle, is_cancelled)
    }

    fn accept_until_cancelled(
        &self,
        handle: u64,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<Transport, String> {
        let listener = self.listener(handle)?;
        loop {
            if is_cancelled() {
                return Err("Network accept cancelled".to_string());
            }
            if listener.is_closed() {
                return Err("Network listener is closed or does not belong to this VM".to_string());
            }
            let (stream, _) = match listener.socket.accept() {
                Ok(accepted) => accepted,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::park_timeout(Duration::from_millis(10));
                    continue;
                }
                Err(error) => return Err(format!("Network listener accept failed: {error}")),
            };
            if is_cancelled() {
                return Err("Network accept cancelled".to_string());
            }
            if listener.is_closed() {
                return Err("Network listener is closed or does not belong to this VM".to_string());
            }
            stream
                .set_nodelay(true)
                .map_err(|error| format!("Could not configure accepted connection: {error}"))?;
            match &listener.mode {
                ListenerMode::Tcp => return Ok(Transport::tcp(stream)),
                ListenerMode::Tls(tls) => {
                    if let Ok(stream) =
                        tls.accept(stream, || listener.is_closed() || is_cancelled())
                    {
                        return Ok(Transport::tls_server(stream));
                    }
                }
            }
        }
    }

    /// Close and invalidate one listener handle.
    pub(super) fn close(&self, handle: u64) -> Result<(), String> {
        validate_handle(handle)?;
        let listener = self
            .listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&handle)
            .ok_or_else(|| {
                "Network listener is closed or does not belong to this VM".to_string()
            })?;
        listener.close();
        Ok(())
    }

    /// Interrupt and invalidate every listener owned by the VM.
    pub(in crate::vm::hosted) fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        let listeners = self
            .listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .drain()
            .map(|(_, listener)| listener)
            .collect::<Vec<_>>();
        for listener in listeners {
            listener.close();
        }
    }

    fn listener(&self, handle: u64) -> Result<Arc<Listener>, String> {
        validate_handle(handle)?;
        self.listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&handle)
            .cloned()
            .ok_or_else(|| "Network listener is closed or does not belong to this VM".to_string())
    }
}

fn listener_port(port: i64) -> Result<u16, String> {
    u16::try_from(port)
        .ok()
        .filter(|port| *port != 0)
        .ok_or_else(|| format!("Network listener port must be in 1..=65535, got {port}"))
}

fn validate_handle(handle: u64) -> Result<(), String> {
    if handle & HANDLE_TAG_MASK == HANDLE_TAG {
        Ok(())
    } else {
        Err("Value is not a network listener handle".to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, mpsc};
    use std::time::Duration;

    use super::NetworkListeners;

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("reserve local port");
        listener.local_addr().expect("reserved address").port()
    }

    #[test]
    fn listener_accepts_one_tcp_connection() {
        let port = unused_port();
        let listeners = NetworkListeners::new();
        let handle = listeners
            .listen("127.0.0.1", i64::from(port))
            .expect("listen");
        let client = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
            stream.write_all(b"ping").expect("write request");
            let mut response = [0_u8; 4];
            stream.read_exact(&mut response).expect("read response");
            response
        });

        let mut stream = listeners.accept(handle).expect("accept");
        let mut request = [0_u8; 4];
        stream.read_exact(&mut request).expect("read request");
        stream.write_all(b"pong").expect("write response");

        assert_eq!(request, *b"ping");
        assert_eq!(client.join().expect("join client"), *b"pong");
    }

    #[test]
    fn closed_listener_cannot_accept() {
        let port = unused_port();
        let listeners = NetworkListeners::new();
        let handle = listeners
            .listen("127.0.0.1", i64::from(port))
            .expect("listen");

        listeners.close(handle).expect("close");

        assert!(listeners.accept(handle).is_err());
    }

    #[test]
    fn invalid_tls_configuration_does_not_bind_socket() {
        let port = unused_port();
        let listeners = NetworkListeners::new();

        let error = listeners
            .listen_tls(
                "127.0.0.1",
                i64::from(port),
                "missing-certificate.pem",
                "missing-private-key.pem",
                1_000,
            )
            .expect_err("missing TLS files must fail");

        assert!(error.contains("certificate"));
        TcpListener::bind(("127.0.0.1", port))
            .expect("invalid TLS configuration must not bind the socket");
    }

    #[test]
    fn close_interrupts_a_blocked_accept() {
        let port = unused_port();
        let listeners = Arc::new(NetworkListeners::new());
        let handle = listeners
            .listen("127.0.0.1", i64::from(port))
            .expect("listen");
        let accept_listeners = Arc::clone(&listeners);
        let (result_sender, result_receiver) = mpsc::channel();
        let accept = std::thread::spawn(move || {
            result_sender
                .send(accept_listeners.accept(handle))
                .expect("send accept result");
        });
        std::thread::sleep(Duration::from_millis(50));

        listeners.close(handle).expect("close");
        let result = result_receiver.recv_timeout(Duration::from_millis(200));
        if result.is_err() {
            drop(TcpStream::connect(("127.0.0.1", port)));
        }
        accept.join().expect("join accept");

        let error = match result.expect("close must interrupt the blocked accept") {
            Ok(_) => panic!("closed listener must not accept a connection"),
            Err(error) => error,
        };
        assert!(error.contains("closed"), "unexpected error: {error}");
    }

    #[test]
    fn cancellation_interrupts_a_blocked_accept_without_closing_the_listener() {
        let port = unused_port();
        let listeners = Arc::new(NetworkListeners::new());
        let handle = listeners
            .listen("127.0.0.1", i64::from(port))
            .expect("listen");
        let cancelled = Arc::new(AtomicBool::new(false));
        let accept_listeners = Arc::clone(&listeners);
        let accept_cancelled = Arc::clone(&cancelled);
        let (result_sender, result_receiver) = mpsc::channel();
        let accept =
            std::thread::spawn(move || {
                result_sender
                    .send(accept_listeners.accept_with_cancellation(handle, || {
                        accept_cancelled.load(Ordering::Acquire)
                    }))
                    .expect("send accept result");
            });
        std::thread::sleep(Duration::from_millis(50));

        cancelled.store(true, Ordering::Release);
        let result = result_receiver
            .recv_timeout(Duration::from_millis(200))
            .expect("cancellation must interrupt the blocked accept");
        accept.join().expect("join accept");

        let error = match result {
            Ok(_) => panic!("cancelled accept must fail"),
            Err(error) => error,
        };
        assert_eq!(error, "Network accept cancelled");
        listeners.close(handle).expect("listener remains open");
    }
}
