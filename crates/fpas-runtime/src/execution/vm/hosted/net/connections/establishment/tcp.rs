//! One non-blocking TCP attempt whose socket is owned until success or cancellation.
//!
//! Documentation: `docs/pascal/std/network/net.md`.

use crate::execution::vm::hosted::net::failure::Failure;
use std::io;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use mio::{Events, Interest, Poll, Token};

use super::remaining;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Connect once, checking cancellation and the shared deadline while the socket is pending.
pub(super) fn connect(
    address: SocketAddr,
    deadline: Instant,
    is_cancelled: impl Fn() -> bool,
) -> Result<TcpStream, Failure> {
    remaining(deadline, &is_cancelled)?;
    let mut poll = Poll::new().map_err(connect_error)?;
    let mut socket = mio::net::TcpStream::connect(address).map_err(connect_error)?;
    poll.registry()
        .register(&mut socket, Token(0), Interest::WRITABLE)
        .map_err(connect_error)?;
    let mut events = Events::with_capacity(4);
    wait_for_connection(deadline, &is_cancelled, || {
        events.clear();
        match poll.poll(&mut events, Some(Duration::ZERO)) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => return Ok(false),
            Err(error) => return Err(error),
            Ok(()) => {}
        }
        // Windows peer_addr can precede completion of the socket's connect context.
        if events.is_empty() {
            return Ok(false);
        }
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
    poll.registry()
        .deregister(&mut socket)
        .map_err(connect_error)?;
    let stream: TcpStream = socket.into();
    stream.set_nonblocking(false).map_err(connect_error)?;
    remaining(deadline, is_cancelled)?;
    Ok(stream)
}

fn wait_for_connection(
    deadline: Instant,
    is_cancelled: impl Fn() -> bool,
    mut check: impl FnMut() -> io::Result<bool>,
) -> Result<(), Failure> {
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

fn connect_error(error: io::Error) -> Failure {
    Failure::io(&error, format!("TCP connect failed: {error}"))
}

#[cfg(test)]
mod tests;
