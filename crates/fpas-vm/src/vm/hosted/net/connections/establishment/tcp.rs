//! One non-blocking TCP attempt whose socket is owned until success or cancellation.
//!
//! Documentation: `docs/pascal/std/network/net.md`.

use std::io;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use socket2::{Domain, Protocol, Socket, Type};

use super::remaining;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Connect once, checking cancellation and the shared deadline while the socket is pending.
pub(super) fn connect(
    address: SocketAddr,
    deadline: Instant,
    is_cancelled: impl Fn() -> bool,
) -> Result<TcpStream, String> {
    remaining(deadline, &is_cancelled)?;
    let socket = Socket::new(
        Domain::for_address(address),
        Type::STREAM,
        Some(Protocol::TCP),
    )
    .map_err(connect_error)?;
    socket.set_nonblocking(true).map_err(connect_error)?;
    match socket.connect(&address.into()) {
        Ok(()) => {}
        Err(error) if in_progress(&error) => {}
        Err(error) => return Err(connect_error(error)),
    }
    wait_for_connection(deadline, &is_cancelled, || {
        if let Some(error) = socket.take_error()? {
            return Err(error);
        }
        match socket.peer_addr() {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotConnected || in_progress(&error) => {
                Ok(false)
            }
            Err(error) => Err(error),
        }
    })?;
    socket.set_nonblocking(false).map_err(connect_error)?;
    remaining(deadline, is_cancelled)?;
    Ok(socket.into())
}

fn wait_for_connection(
    deadline: Instant,
    is_cancelled: impl Fn() -> bool,
    mut check: impl FnMut() -> io::Result<bool>,
) -> Result<(), String> {
    loop {
        remaining(deadline, &is_cancelled)?;
        let ready = check();
        let left = remaining(deadline, &is_cancelled)?;
        if ready.map_err(connect_error)? {
            return Ok(());
        }
        std::thread::park_timeout(POLL_INTERVAL.min(left));
    }
}

fn in_progress(error: &io::Error) -> bool {
    if matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
    ) {
        return true;
    }
    #[cfg(unix)]
    if error.raw_os_error() == Some(libc::EINPROGRESS) {
        return true;
    }
    false
}

fn connect_error(error: io::Error) -> String {
    format!("TCP connect failed: {error}")
}

#[cfg(test)]
mod tests;
