//! Close notifications advance without retaining an I/O thread or a registry entry.

use super::{Failure, NetworkConnections, Transport, validate_handle};
use std::{
    io::{self, Write},
    sync::TryLockError,
    task::Poll,
    time::{Duration, Instant},
};

#[cfg(test)]
mod tests;

pub(in crate::execution::vm::hosted::net) struct Closing {
    transport: Transport,
    deadline: Option<Instant>,
}

impl NetworkConnections {
    pub(in crate::execution::vm::hosted::net) fn begin_close(
        &self,
        handle: u64,
    ) -> Result<Closing, Failure> {
        validate_handle(handle)?;
        let connection = self
            .connections
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&handle)
            .ok_or(Failure::Closed)?;
        let transport = match connection.transport.try_lock() {
            Ok(mut transport) => std::mem::replace(&mut *transport, Transport::Closed),
            Err(TryLockError::Poisoned(error)) => {
                std::mem::replace(&mut *error.into_inner(), Transport::Closed)
            }
            Err(TryLockError::WouldBlock) => {
                connection.interrupt().map_err(close_error)?;
                return Ok(Closing {
                    transport: Transport::Closed,
                    deadline: None,
                });
            }
        };
        Closing::new(transport)
    }

    /// Unpublished resources have no application bytes to preserve.
    pub(in crate::execution::vm::hosted::net) fn abort(&self, handle: u64) -> Result<(), Failure> {
        validate_handle(handle)?;
        let connection = self
            .connections
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&handle)
            .ok_or(Failure::Closed)?;
        connection.interrupt().map_err(close_error)
    }
}

impl Closing {
    fn new(transport: Transport) -> Result<Self, Failure> {
        let mut closing = Self {
            transport,
            deadline: None,
        };
        closing.deadline = closing
            .transport
            .write_timeout()
            .map_err(close_error)?
            .and_then(|timeout| Instant::now().checked_add(timeout));
        closing
            .transport
            .set_nonblocking(true)
            .map_err(close_error)?;
        closing.transport.begin_shutdown().map_err(close_error)?;
        Ok(closing)
    }

    pub(in crate::execution::vm::hosted::net) fn poll(&mut self) -> Poll<Result<(), Failure>> {
        if matches!(self.transport, Transport::Closed) {
            return Poll::Ready(Ok(()));
        }
        match self.transport.flush() {
            Ok(()) => Poll::Ready(self.transport.abort().map_err(close_error)),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                if self
                    .deadline
                    .is_some_and(|deadline| Instant::now() >= deadline)
                {
                    Poll::Ready(Err(Failure::deadline("Network close timed out")))
                } else {
                    Poll::Pending
                }
            }
            Err(error) => Poll::Ready(Err(close_error(error))),
        }
    }

    pub(super) fn finish(mut self) -> Result<(), Failure> {
        loop {
            match self.poll() {
                Poll::Ready(result) => return result,
                Poll::Pending => std::thread::park_timeout(Duration::from_millis(1)),
            }
        }
    }
}

impl Drop for Closing {
    fn drop(&mut self) {
        let _ = self.transport.abort();
    }
}

fn close_error(error: io::Error) -> Failure {
    Failure::io(&error, format!("Network connection close failed: {error}"))
}
