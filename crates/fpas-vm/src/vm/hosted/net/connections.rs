//! VM-owned network connection registry.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, TryLockError};
use std::time::Duration;

mod cancellable_read;
mod cancellable_write;
mod establishment;
mod polling;
#[cfg(test)]
mod test_connections;
mod tls;
use super::transport::Transport;
pub(super) use establishment::ConnectMode;

const HANDLE_TAG: u64 = 0x4E45_0000_0000_0000;
const HANDLE_TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const MAX_IO_BYTES: usize = 1024 * 1024;
const MAX_TIMEOUT_MILLIS: u64 = 300_000;

type SharedConnection = Arc<Connection>;

struct Connection {
    transport: Mutex<Transport>,
    interrupt: TcpStream,
}

impl Connection {
    fn new(transport: Transport) -> Result<Self, String> {
        let interrupt = transport
            .try_clone_socket()
            .map_err(|error| format!("Could not prepare network connection shutdown: {error}"))?;
        Ok(Self {
            transport: Mutex::new(transport),
            interrupt,
        })
    }

    fn interrupt(&self) -> std::io::Result<()> {
        self.interrupt.shutdown(Shutdown::Both).or_else(|error| {
            if error.kind() == std::io::ErrorKind::NotConnected {
                Ok(())
            } else {
                Err(error)
            }
        })
    }

    fn close(&self) -> std::io::Result<()> {
        match self.transport.try_lock() {
            Ok(mut transport) => transport.shutdown(),
            Err(TryLockError::WouldBlock) => self.interrupt(),
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner().shutdown(),
        }
    }
}

/// Thread-safe TCP and TLS resources owned by one VM.
pub(in crate::vm::hosted) struct NetworkConnections {
    next_handle: AtomicU64,
    connections: Mutex<HashMap<u64, SharedConnection>>,
    shutdown: AtomicBool,
}

impl NetworkConnections {
    /// Create an empty per-VM connection registry.
    pub(in crate::vm::hosted) fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(HANDLE_TAG | 1),
            connections: Mutex::new(HashMap::new()),
            shutdown: AtomicBool::new(false),
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
        self.insert(Transport::tcp(stream))
    }

    /// Store an accepted TCP or TLS connection and return its opaque runtime handle.
    pub(super) fn insert_accepted(&self, transport: Transport) -> Result<u64, String> {
        self.insert(transport)
    }

    /// Set both read and write timeouts; zero disables them.
    pub(super) fn set_timeout(&self, handle: u64, timeout_millis: i64) -> Result<(), String> {
        let duration = timeout(timeout_millis, true)?;
        let connection = self.connection(handle)?;
        let transport = connection
            .transport
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        transport
            .set_timeout((!duration.is_zero()).then_some(duration))
            .map_err(|error| format!("Could not configure network connection timeout: {error}"))
    }

    /// Read at most `max_bytes`; an empty result means end of stream.
    pub(super) fn read(&self, handle: u64, max_bytes: i64) -> Result<Vec<u8>, String> {
        let max_bytes = read_size(max_bytes)?;
        let connection = self.connection(handle)?;
        let mut transport = connection
            .transport
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
        write_size(bytes.len())?;
        let connection = self.connection(handle)?;
        let mut transport = connection
            .transport
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        transport
            .write(bytes)
            .map_err(|error| format!("{} write failed: {error}", transport.name()))
    }

    /// Close and invalidate one connection handle.
    pub(super) fn close(&self, handle: u64) -> Result<(), String> {
        validate_handle(handle)?;
        let connection = self
            .connections
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&handle)
            .ok_or_else(|| {
                "Network connection is closed or does not belong to this VM".to_string()
            })?;
        connection
            .close()
            .map_err(|error| format!("Network connection close failed: {error}"))
    }

    /// Interrupt and invalidate every connection owned by the VM.
    pub(in crate::vm::hosted) fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        let connections = self
            .connections
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .drain()
            .map(|(_, connection)| connection)
            .collect::<Vec<_>>();
        for connection in connections {
            drop(connection.interrupt());
        }
    }

    fn insert(&self, transport: Transport) -> Result<u64, String> {
        let connection = Arc::new(Connection::new(transport)?);
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        let mut connections = self
            .connections
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if self.shutdown.load(Ordering::Acquire) {
            drop(connections);
            drop(connection.interrupt());
            return Err("Network connection cannot be opened after VM shutdown".to_string());
        }
        connections.insert(handle, connection);
        Ok(handle)
    }

    fn connection(&self, handle: u64) -> Result<SharedConnection, String> {
        validate_handle(handle)?;
        self.connections
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
    let port = connect_port(port)?;
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

fn read_size(max_bytes: i64) -> Result<usize, String> {
    usize::try_from(max_bytes)
        .ok()
        .filter(|size| (1..=MAX_IO_BYTES).contains(size))
        .ok_or_else(|| format!("Network read size must be in 1..={MAX_IO_BYTES}, got {max_bytes}"))
}

fn connect_port(port: i64) -> Result<u16, String> {
    u16::try_from(port)
        .ok()
        .filter(|port| *port != 0)
        .ok_or_else(|| format!("TCP port must be in 1..=65535, got {port}"))
}

fn validate_handle(handle: u64) -> Result<(), String> {
    if handle & HANDLE_TAG_MASK == HANDLE_TAG {
        Ok(())
    } else {
        Err("Value is not a network connection handle".to_string())
    }
}

fn write_size(size: usize) -> Result<(), String> {
    if size > MAX_IO_BYTES {
        Err(format!(
            "Network write size must not exceed {MAX_IO_BYTES} bytes, got {size}"
        ))
    } else {
        Ok(())
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
mod tests;
