//! VM-owned TCP connection registry.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const HANDLE_TAG: u64 = 0x4E45_0000_0000_0000;
const HANDLE_TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const MAX_IO_BYTES: usize = 1024 * 1024;
const MAX_TIMEOUT_MILLIS: u64 = 300_000;

type SharedStream = Arc<Mutex<TcpStream>>;

/// Thread-safe TCP resources owned by one VM.
pub(in crate::vm::hosted) struct TcpConnections {
    next_handle: AtomicU64,
    streams: Mutex<HashMap<u64, SharedStream>>,
}

impl TcpConnections {
    /// Create an empty per-VM connection registry.
    pub(in crate::vm::hosted) fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(HANDLE_TAG | 1),
            streams: Mutex::new(HashMap::new()),
        }
    }

    /// Open a TCP connection and return its opaque runtime handle.
    pub(super) fn connect(
        &self,
        host: &str,
        port: i64,
        timeout_millis: i64,
    ) -> Result<u64, String> {
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
                    let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
                    self.streams
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .insert(handle, Arc::new(Mutex::new(stream)));
                    return Ok(handle);
                }
                Err(error) => last_error = Some(error),
            }
        }

        Err(last_error.map_or_else(
            || format!("TCP host '{host}' resolved to no addresses"),
            |error| format!("Could not connect to {host}:{port}: {error}"),
        ))
    }

    /// Set both read and write timeouts; zero disables them.
    pub(super) fn set_timeout(&self, handle: u64, timeout_millis: i64) -> Result<(), String> {
        let duration = timeout(timeout_millis, true)?;
        let stream = self.stream(handle)?;
        let stream = stream
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let duration = (!duration.is_zero()).then_some(duration);
        stream
            .set_read_timeout(duration)
            .and_then(|()| stream.set_write_timeout(duration))
            .map_err(|error| format!("Could not configure TCP timeout: {error}"))
    }

    /// Read at most `max_bytes`; an empty result means end of stream.
    pub(super) fn read(&self, handle: u64, max_bytes: i64) -> Result<Vec<u8>, String> {
        let max_bytes = usize::try_from(max_bytes)
            .ok()
            .filter(|size| (1..=MAX_IO_BYTES).contains(size))
            .ok_or_else(|| {
                format!("TCP read size must be in 1..={MAX_IO_BYTES}, got {max_bytes}")
            })?;
        let stream = self.stream(handle)?;
        let mut stream = stream
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut bytes = vec![0; max_bytes];
        let count = stream
            .read(&mut bytes)
            .map_err(|error| format!("TCP read failed: {error}"))?;
        bytes.truncate(count);
        Ok(bytes)
    }

    /// Write at most one bounded byte chunk and return the number written.
    pub(super) fn write(&self, handle: u64, bytes: &[u8]) -> Result<usize, String> {
        if bytes.len() > MAX_IO_BYTES {
            return Err(format!(
                "TCP write size must not exceed {MAX_IO_BYTES} bytes, got {}",
                bytes.len()
            ));
        }
        let stream = self.stream(handle)?;
        stream
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .write(bytes)
            .map_err(|error| format!("TCP write failed: {error}"))
    }

    /// Close and invalidate one connection handle.
    pub(super) fn close(&self, handle: u64) -> Result<(), String> {
        validate_handle(handle)?;
        let stream = self
            .streams
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&handle)
            .ok_or_else(|| "TCP connection is closed or does not belong to this VM".to_string())?;
        stream
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .shutdown(Shutdown::Both)
            .or_else(|error| {
                if error.kind() == std::io::ErrorKind::NotConnected {
                    Ok(())
                } else {
                    Err(error)
                }
            })
            .map_err(|error| format!("TCP close failed: {error}"))
    }

    fn stream(&self, handle: u64) -> Result<SharedStream, String> {
        validate_handle(handle)?;
        self.streams
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&handle)
            .cloned()
            .ok_or_else(|| "TCP connection is closed or does not belong to this VM".to_string())
    }
}

fn validate_handle(handle: u64) -> Result<(), String> {
    if handle & HANDLE_TAG_MASK == HANDLE_TAG {
        Ok(())
    } else {
        Err("Value is not a TCP connection handle".to_string())
    }
}

fn timeout(millis: i64, allow_zero: bool) -> Result<Duration, String> {
    let millis = u64::try_from(millis)
        .ok()
        .filter(|millis| (*millis != 0 || allow_zero) && *millis <= MAX_TIMEOUT_MILLIS)
        .ok_or_else(|| {
            let minimum = if allow_zero { 0 } else { 1 };
            format!("TCP timeout must be in {minimum}..={MAX_TIMEOUT_MILLIS} ms, got {millis}")
        })?;
    Ok(Duration::from_millis(millis))
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    use super::TcpConnections;

    #[test]
    fn connection_round_trip_preserves_bytes() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local listener");
        let address = listener.local_addr().expect("listener address");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept client");
            let mut request = [0_u8; 4];
            stream.read_exact(&mut request).expect("read request");
            assert_eq!(&request, b"ping");
            stream.write_all(b"pong").expect("write response");
        });

        let connections = TcpConnections::new();
        let handle = connections
            .connect("127.0.0.1", i64::from(address.port()), 1_000)
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

        let connections = TcpConnections::new();
        let handle = connections
            .connect("127.0.0.1", i64::from(address.port()), 1_000)
            .expect("connect client");
        connections.close(handle).expect("close");
        assert!(connections.read(handle, 1).is_err());
        drop(server.join().expect("join server"));
    }
}
