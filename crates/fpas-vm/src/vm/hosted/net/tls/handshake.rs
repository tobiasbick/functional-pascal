//! Bounded TLS handshake I/O shared by client and listener setup.

use std::io;
use std::net::TcpStream;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Complete handshake steps while observing the caller's cancellation state.
pub(super) fn complete(
    socket: &mut TcpStream,
    timeout: Duration,
    is_cancelled: impl Fn() -> bool,
    mut advance: impl FnMut(&mut TcpStream) -> io::Result<bool>,
) -> Result<(), String> {
    let interval = timeout.min(POLL_INTERVAL);
    socket
        .set_read_timeout(Some(interval))
        .and_then(|()| socket.set_write_timeout(Some(interval)))
        .map_err(|error| format!("Could not configure TLS handshake timeout: {error}"))?;
    let deadline = Instant::now() + timeout;
    loop {
        if is_cancelled() {
            return Err("Network operation cancelled during TLS handshake".to_string());
        }
        match advance(socket) {
            Ok(true) => break,
            Ok(false) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(error) => return Err(format!("TLS handshake failed: {error}")),
        }
        if Instant::now() >= deadline {
            return Err("TLS handshake timed out".to_string());
        }
    }
    socket
        .set_read_timeout(None)
        .and_then(|()| socket.set_write_timeout(None))
        .map_err(|error| format!("Could not clear TLS handshake timeout: {error}"))
}
