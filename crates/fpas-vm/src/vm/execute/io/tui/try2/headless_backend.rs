//! Try-2 in-memory turbo-vision [`Backend`] for headless FPAS tests.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use turbo_vision::core::event::Event;
use turbo_vision::terminal::Backend;

/// Queues synthetic input for a paired [`TvHeadlessBackend`].
pub(in crate::vm::execute::io::tui) struct HeadlessTvEventInbox {
    events: Arc<Mutex<Vec<Event>>>,
}

impl HeadlessTvEventInbox {
    /// Queues an input event for the next `poll_event` call on the paired backend.
    pub fn push(&self, event: Event) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }
}

/// Fixed-size terminal backend with a queued event inbox for headless tests.
pub(in crate::vm::execute::io::tui) struct TvHeadlessBackend {
    width: u16,
    height: u16,
    events: Arc<Mutex<Vec<Event>>>,
}

impl TvHeadlessBackend {
    /// Creates a backend and the inbox used to queue `TestClickMouse` / keyboard events.
    pub fn new(width: u16, height: u16) -> (Self, HeadlessTvEventInbox) {
        let events = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                width,
                height,
                events: Arc::clone(&events),
            },
            HeadlessTvEventInbox { events },
        )
    }
}

impl Backend for TvHeadlessBackend {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn init(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn size(&self) -> io::Result<(u16, u16)> {
        Ok((self.width, self.height))
    }

    fn poll_event(&mut self, _timeout: Duration) -> io::Result<Option<Event>> {
        let mut events = self
            .events
            .lock()
            .map_err(|_| io::Error::other("headless backend event queue poisoned"))?;
        Ok(if events.is_empty() {
            None
        } else {
            Some(events.remove(0))
        })
    }

    fn write_raw(&mut self, _data: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn show_cursor(&mut self, _x: u16, _y: u16) -> io::Result<()> {
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }
}
