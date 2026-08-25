//! VM-owned network connection registry.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::tls::client;
use super::transport::Transport;

const HANDLE_TAG: u64 = 0x4E45_0000_0000_0000;
const HANDLE_TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const MAX_IO_BYTES: usize = 1024 * 1024;
const MAX_TIMEOUT_MILLIS: u64 = 300_000;

type SharedTransport = Arc<Mutex<Transport>>;

/// Thread-safe TCP and TLS resources owned by one VM.
pub(in crate::vm::hosted) struct NetworkConnections {
    next_handle: AtomicU64,
    transports: Mutex<HashMap<u64, SharedTransport>>,
}

impl NetworkConnections {
    /// Create an empty per-VM connection registry.
    pub(in crate::vm::hosted) fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(HANDLE_TAG | 1),
            transports: Mutex::new(HashMap::new()),
        }
    }

    /// Open a TCP connection and return its opaque runtime handle.
    pub(super) fn connect_tcp(
        &self,
        host: &str,
        port: i64,
        timeout_millis: i64,
    ) -> Result<u64, String> {
        let (stream, _) = connect_socket(host, port, timeout_millis)?;
        Ok(self.insert(Transport::tcp(stream)))
    }

    /// Store an accepted TCP or TLS connection and return its opaque runtime handle.
    pub(super) fn insert_accepted(&self, transport: Transport) -> u64 {
        self.insert(transport)
    }

    /// Open a verified TLS connection and return its opaque runtime handle.
    pub(super) fn connect_tls(
        &self,
        host: &str,
        port: i64,
        timeout_millis: i64,
    ) -> Result<u64, String> {
        let (stream, timeout) = connect_socket(host, port, timeout_millis)?;
        stream
            .set_read_timeout(Some(timeout))
            .and_then(|()| stream.set_write_timeout(Some(timeout)))
            .map_err(|error| format!("Could not configure TLS timeout: {error}"))?;
        let stream = client::connect(stream, host)?;
        stream
            .sock
            .set_read_timeout(None)
            .and_then(|()| stream.sock.set_write_timeout(None))
            .map_err(|error| format!("Could not clear TLS handshake timeout: {error}"))?;
        Ok(self.insert(Transport::tls_client(stream)))
    }

    /// Set both read and write timeouts; zero disables them.
    pub(super) fn set_timeout(&self, handle: u64, timeout_millis: i64) -> Result<(), String> {
        let duration = timeout(timeout_millis, true)?;
        let transport = self.transport(handle)?;
        let transport = transport
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        transport
            .set_timeout((!duration.is_zero()).then_some(duration))
            .map_err(|error| format!("Could not configure network connection timeout: {error}"))
    }

    /// Read at most `max_bytes`; an empty result means end of stream.
    pub(super) fn read(&self, handle: u64, max_bytes: i64) -> Result<Vec<u8>, String> {
        let max_bytes = usize::try_from(max_bytes)
            .ok()
            .filter(|size| (1..=MAX_IO_BYTES).contains(size))
            .ok_or_else(|| {
                format!("Network read size must be in 1..={MAX_IO_BYTES}, got {max_bytes}")
            })?;
        let transport = self.transport(handle)?;
        let mut transport = transport
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut bytes = vec![0; max_bytes];
        let count = transport
            .read(&mut bytes)
            .map_err(|error| format!("{} read failed: {error}", transport.name()))?;
        bytes.truncate(count);
        Ok(bytes)
    }

    /// Write at most one bounded byte chunk and return the number written.
    pub(super) fn write(&self, handle: u64, bytes: &[u8]) -> Result<usize, String> {
        if bytes.len() > MAX_IO_BYTES {
            return Err(format!(
                "Network write size must not exceed {MAX_IO_BYTES} bytes, got {}",
                bytes.len()
            ));
        }
        let transport = self.transport(handle)?;
        let mut transport = transport
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        transport
            .write(bytes)
            .map_err(|error| format!("{} write failed: {error}", transport.name()))
    }

    /// Close and invalidate one connection handle.
    pub(super) fn close(&self, handle: u64) -> Result<(), String> {
        validate_handle(handle)?;
        let transport = self
            .transports
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&handle)
            .ok_or_else(|| {
                "Network connection is closed or does not belong to this VM".to_string()
            })?;
        transport
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .shutdown()
            .map_err(|error| format!("Network connection close failed: {error}"))
    }

    fn insert(&self, transport: Transport) -> u64 {
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        self.transports
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(handle, Arc::new(Mutex::new(transport)));
        handle
    }

    fn transport(&self, handle: u64) -> Result<SharedTransport, String> {
        validate_handle(handle)?;
        self.transports
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&handle)
            .cloned()
            .ok_or_else(|| "Network connection is closed or does not belong to this VM".to_string())
    }
}

fn connect_socket(
    host: &str,
    port: i64,
    timeout_millis: i64,
) -> Result<(TcpStream, Duration), String> {
    let port = u16::try_from(port)
        .ok()
        .filter(|port| *port != 0)
        .ok_or_else(|| format!("TCP port must be in 1..=65535, got {port}"))?;
    let timeout = timeout(timeout_millis, false)?;
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|error| format!("Could not resolve TCP host '{host}': {error}"))?;

    let mut last_error = None;
    for address in addresses {
        match TcpStream::connect_timeout(&address, timeout) {
            Ok(stream) => {
                stream
                    .set_nodelay(true)
                    .map_err(|error| format!("Could not configure TCP connection: {error}"))?;
                return Ok((stream, timeout));
            }
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.map_or_else(
        || format!("TCP host '{host}' resolved to no addresses"),
        |error| format!("Could not connect to {host}:{port}: {error}"),
    ))
}

fn validate_handle(handle: u64) -> Result<(), String> {
    if handle & HANDLE_TAG_MASK == HANDLE_TAG {
        Ok(())
    } else {
        Err("Value is not a network connection handle".to_string())
    }
}

fn timeout(millis: i64, allow_zero: bool) -> Result<Duration, String> {
    let millis = u64::try_from(millis)
        .ok()
        .filter(|millis| (*millis != 0 || allow_zero) && *millis <= MAX_TIMEOUT_MILLIS)
        .ok_or_else(|| {
            let minimum = if allow_zero { 0 } else { 1 };
            format!("Network timeout must be in {minimum}..={MAX_TIMEOUT_MILLIS} ms, got {millis}")
        })?;
    Ok(Duration::from_millis(millis))
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    use super::NetworkConnections;

    #[test]
    fn tcp_connection_round_trip_preserves_bytes() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local listener");
        let address = listener.local_addr().expect("listener address");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept client");
            let mut request = [0_u8; 4];
            stream.read_exact(&mut request).expect("read request");
            assert_eq!(&request, b"ping");
            stream.write_all(b"pong").expect("write response");
        });

        let connections = NetworkConnections::new();
        let handle = connections
            .connect_tcp("127.0.0.1", i64::from(address.port()), 1_000)
            .expect("connect client");
        assert_eq!(connections.write(handle, b"ping").expect("write"), 4);
        assert_eq!(connections.read(handle, 16).expect("read"), b"pong");
        connections.close(handle).expect("close");
        server.join().expect("join server");
    }

    #[test]
    fn closed_connection_cannot_be_reused() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local listener");
        let address = listener.local_addr().expect("listener address");
        let server = std::thread::spawn(move || listener.accept().expect("accept client"));

        let connections = NetworkConnections::new();
        let handle = connections
            .connect_tcp("127.0.0.1", i64::from(address.port()), 1_000)
            .expect("connect client");
        connections.close(handle).expect("close");
        assert!(connections.read(handle, 1).is_err());
        drop(server.join().expect("join server"));
    }
}
