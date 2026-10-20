//! Serialized non-blocking I/O with cancellation and per-operation timeouts.

use crate::vm::hosted::net::failure::Failure;
use std::io;
use std::sync::TryLockError;
use std::time::{Duration, Instant};

use super::{NetworkConnections, Transport};

const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Socket timeout and diagnostic direction for one polling operation.
#[derive(Clone, Copy)]
pub(super) enum Direction {
    /// Reading application bytes.
    Read,
    /// Writing application bytes.
    Write,
}

impl NetworkConnections {
    /// Retry an operation only while it has not reported any progress.
    pub(super) fn poll_io(
        &self,
        handle: u64,
        direction: Direction,
        is_cancelled: impl Fn() -> bool,
        mut attempt: impl FnMut(&mut Transport) -> io::Result<usize>,
    ) -> Result<usize, Failure> {
        let name = match direction {
            Direction::Read => "read",
            Direction::Write => "write",
        };
        let connection = self.connection(handle)?;
        let mut transport = loop {
            if is_cancelled() {
                return Err(Failure::Cancelled);
            }
            match connection.transport.try_lock() {
                Ok(transport) => break transport,
                Err(TryLockError::Poisoned(error)) => break error.into_inner(),
                Err(TryLockError::WouldBlock) => std::thread::park_timeout(POLL_INTERVAL),
            }
        };
        let timeout = match direction {
            Direction::Read => transport.read_timeout(),
            Direction::Write => transport.write_timeout(),
        }
        .map_err(|error| {
            Failure::io(
                &error,
                format!("Could not inspect network {name} timeout: {error}"),
            )
        })?;
        transport.set_nonblocking(true).map_err(|error| {
            Failure::io(
                &error,
                format!("Could not configure cancellable network {name}: {error}"),
            )
        })?;
        let started = Instant::now();
        let result = loop {
            if is_cancelled() {
                break Err(Failure::Cancelled);
            }
            if timeout.is_some_and(|timeout| started.elapsed() >= timeout) {
                break Err(Failure::deadline(format!("Network {name} timed out")));
            }
            match attempt(&mut transport) {
                Ok(count) => break Ok(count),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    let wait = timeout.map_or(POLL_INTERVAL, |timeout| {
                        POLL_INTERVAL.min(timeout.saturating_sub(started.elapsed()))
                    });
                    std::thread::park_timeout(wait);
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => {
                    break Err(Failure::io(
                        &error,
                        format!("{} {name} failed: {error}", transport.name()),
                    ));
                }
            }
        };
        // Never hide bytes already consumed by reporting a later restoration error instead.
        if let Err(error) = transport.set_nonblocking(false) {
            drop(self.close(handle));
            if result.is_err() {
                return Err(Failure::io(
                    &error,
                    format!("Could not restore blocking network {name}: {error}"),
                ));
            }
        }
        result
    }
}
