//! Terminal event source boundary for live reads and deterministic unit tests.

use std::io;
use std::sync::{Mutex, MutexGuard, TryLockError};
use std::time::Duration;
use std::time::Instant;

use crossterm::event::{self, Event};

pub(super) trait TerminalEventSource {
    fn poll(&mut self, timeout: Duration) -> io::Result<bool>;
    fn read(&mut self) -> io::Result<Event>;
}

static EVENTS: Mutex<()> = Mutex::new(());

#[derive(Default)]
pub(super) struct CrosstermEventSource {
    guard: Option<MutexGuard<'static, ()>>,
}

impl TerminalEventSource for CrosstermEventSource {
    fn poll(&mut self, timeout: Duration) -> io::Result<bool> {
        let started = Instant::now();
        while self.guard.is_none() {
            self.guard = match EVENTS.try_lock() {
                Ok(guard) => Some(guard),
                Err(TryLockError::Poisoned(error)) => Some(error.into_inner()),
                Err(TryLockError::WouldBlock) => {
                    if started.elapsed() >= timeout {
                        return Ok(false);
                    }
                    std::thread::park_timeout(
                        Duration::from_millis(1).min(timeout.saturating_sub(started.elapsed())),
                    );
                    None
                }
            };
        }
        let ready = event::poll(timeout.saturating_sub(started.elapsed()));
        if !matches!(ready, Ok(true)) {
            self.guard.take();
        }
        ready
    }

    fn read(&mut self) -> io::Result<Event> {
        // A positive poll retains the process-wide lease until this read finishes.
        let event = event::read();
        self.guard.take();
        event
    }
}
