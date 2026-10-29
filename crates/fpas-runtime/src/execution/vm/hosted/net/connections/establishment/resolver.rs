//! Bounded OS lookups own only native input and addresses, independently of VM lifetime.

use super::remaining;
use crate::execution::vm::hosted::net::failure::Failure;
use std::io;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const MAX_LOOKUPS: usize = 64;
const POLL_INTERVAL: Duration = Duration::from_millis(10);
static ACTIVE: AtomicUsize = AtomicUsize::new(0);

struct Permit<'a>(&'a AtomicUsize);

impl<'a> Permit<'a> {
    fn acquire(active: &'a AtomicUsize) -> Result<Self, Failure> {
        active
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < MAX_LOOKUPS).then_some(count + 1)
            })
            .map_err(|_| {
                Failure::io(
                    &io::Error::other("Pending DNS lookup limit reached"),
                    format!("At most {MAX_LOOKUPS} OS DNS lookups may be pending per process"),
                )
            })?;
        Ok(Self(active))
    }
}

impl Drop for Permit<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

/// A dropped receiver discards late addresses. The OS worker never opens sockets.
struct Lookup {
    receiver: Receiver<io::Result<Vec<SocketAddr>>>,
    thread: Option<JoinHandle<()>>,
}

impl Lookup {
    fn start(
        resolve: impl FnOnce() -> io::Result<Vec<SocketAddr>> + Send + 'static,
    ) -> Result<Self, Failure> {
        let permit = Permit::acquire(&ACTIVE)?;
        let (sender, receiver) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("fpas-dns".into())
            .spawn(move || {
                let _permit = permit;
                let _ = sender.send(resolve());
            })
            .map_err(|error| Failure::io(&error, "Could not start OS DNS lookup".into()))?;
        Ok(Self {
            receiver,
            thread: Some(thread),
        })
    }

    fn wait(
        mut self,
        deadline: Instant,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<Vec<SocketAddr>, Failure> {
        loop {
            let left = remaining(deadline, &is_cancelled)?;
            match self.receiver.recv_timeout(POLL_INTERVAL.min(left)) {
                Ok(outcome) => {
                    // Receiving a result proves the OS call has returned. Join its
                    // native bookkeeping before returning ordinary success or failure.
                    if let Some(thread) = self.thread.take() {
                        let _ = thread.join();
                    }
                    remaining(deadline, &is_cancelled)?;
                    return outcome.map_err(|error| {
                        Failure::resolve(&error, format!("Could not resolve TCP host: {error}"))
                    });
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    remaining(deadline, &is_cancelled)?;
                    return Err(Failure::io(
                        &io::Error::other("DNS worker exited"),
                        "OS DNS lookup exited without a result".into(),
                    ));
                }
            }
        }
    }
}

impl Drop for Lookup {
    fn drop(&mut self) {
        // A resolver stuck inside the OS cannot be joined on cancellation. It
        // retains its process-wide permit until it exits, never any VM state.
        if self.thread.as_ref().is_some_and(JoinHandle::is_finished)
            && let Some(thread) = self.thread.take()
        {
            let _ = thread.join();
        }
    }
}

pub(super) fn resolve(
    host: &str,
    port: u16,
    deadline: Instant,
    is_cancelled: impl Fn() -> bool,
) -> Result<Vec<SocketAddr>, Failure> {
    remaining(deadline, &is_cancelled)?;
    if let Ok(address) = host.parse::<IpAddr>() {
        return Ok(vec![SocketAddr::new(address, port)]);
    }
    let host = host.to_owned();
    Lookup::start(move || {
        (host.as_str(), port)
            .to_socket_addrs()
            .map(Iterator::collect)
    })?
    .wait(deadline, is_cancelled)
}

#[cfg(test)]
mod tests;
