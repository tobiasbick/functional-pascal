//! In-memory turbo-vision [`Backend`] for headless FPAS tests.
//!
//! **Documentation:** `docs/refactor/tui-bridge/done/04-headless-test-util.md`

use std::io;
use std::sync::Mutex;
use std::time::Duration;
use turbo_vision::core::event::Event;
use turbo_vision::terminal::Backend;

/// Fixed-size terminal backend with a queued event inbox for headless tests.
pub(in crate::vm::execute::io::tui) struct TvHeadlessBackend {
    width: u16,
    height: u16,
    events: Mutex<Vec<Event>>,
}

impl TvHeadlessBackend {
    /// Creates a backend reporting `width` × `height` character cells.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            events: Mutex::new(Vec::new()),
        }
    }

    /// Queues an input event for the next `poll_event` call.
    ///
    /// Reserved for routing `TestClickMouse` / keyboard through TV `handle_event`
    /// instead of duplicate hit-test (refactor 03 follow-up).
    #[allow(dead_code)]
    pub fn push_event(&self, event: Event) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
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
