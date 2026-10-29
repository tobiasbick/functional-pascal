//! Bounded TLS handshake I/O shared by client and listener setup.

use crate::execution::vm::hosted::net::failure::Failure;
use std::io;
use std::net::TcpStream;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Complete handshake steps while observing the caller's cancellation state.
pub(super) fn complete(
    socket: &mut TcpStream,
    deadline: Instant,
    is_cancelled: impl Fn() -> bool,
    mut advance: impl FnMut(&mut TcpStream) -> io::Result<bool>,
) -> Result<(), Failure> {
    if is_cancelled() {
        return Err(Failure::Cancelled);
    }
    let interval = deadline
        .checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| Failure::deadline("TLS handshake timed out"))?
        .min(POLL_INTERVAL);
    socket
        .set_read_timeout(Some(interval))
        .and_then(|()| socket.set_write_timeout(Some(interval)))
        .map_err(|error| {
            Failure::io(
                &error,
                format!("Could not configure TLS handshake timeout: {error}"),
            )
        })?;
    loop {
        if is_cancelled() {
            return Err(Failure::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(Failure::deadline("TLS handshake timed out"));
        }
        let result = advance(socket);
        if is_cancelled() {
            return Err(Failure::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(Failure::deadline("TLS handshake timed out"));
        }
        match result {
            Ok(true) => break,
            Ok(false) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(error) => {
                return Err(Failure::io(
                    &error,
                    format!("TLS handshake failed: {error}"),
                ));
            }
        }
    }
    socket
        .set_read_timeout(None)
        .and_then(|()| socket.set_write_timeout(None))
        .map_err(|error| {
            Failure::io(
                &error,
                format!("Could not clear TLS handshake timeout: {error}"),
            )
        })
}
