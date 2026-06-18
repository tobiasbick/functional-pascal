//! Rust-hosted TUI event normalization and coalescing.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

use super::session::TuiSession;
use crate::console::{Console, KeyInput};
use crate::error::StdError;
use crate::ui::{UiEvent, UiHost};
use fpas_bytecode::SourceLocation;

/// Terminal hosted event queue ([`UiHost`] with terminal ingest policy).
pub type TuiHost = UiHost;

impl UiHost {
    /// Non-blocking: returns a ready [`UiEvent`] or polls the session once.
    pub fn poll_next_terminal(
        &mut self,
        session: &TuiSession,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        if let Some(ev) = self.pop_ready_event() {
            return Ok(Some(ev));
        }

        match session.poll_ui_event(console, key_input, location)? {
            None => Ok(None),
            Some(event) => {
                self.ingest_ui_event(event);
                Ok(self.pop_ready_event())
            }
        }
    }

    /// Blocking: read until at least one [`UiEvent`] is returned.
    pub fn read_next_blocking_terminal(
        &mut self,
        session: &TuiSession,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<UiEvent, StdError> {
        loop {
            if let Some(ev) = self.pop_ready_event() {
                return Ok(ev);
            }

            let event = session.read_ui_event(console, key_input, location)?;
            self.ingest_ui_event(event);
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use crate::ConsoleKeyEvent;
    use crate::console_event::ConsoleEvent;
    use crate::key_event::key_kind_index;
    use crate::ui::UiResize;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn coalesces_multiple_resizes_before_key() {
        let mut host = UiHost::for_terminal();
        host.ingest_ui_event(UiEvent::Resize(UiResize::new(Some(80), Some(25), 10, 10)));
        host.ingest_ui_event(UiEvent::Resize(UiResize::new(Some(10), Some(10), 20, 30)));
        host.ingest_ui_event(UiEvent::Key(ConsoleKeyEvent::new(
            key_kind_index("Space"),
            ' ',
            false,
            false,
            false,
            false,
        )));

        assert_eq!(
            host.pop_ready_event(),
            Some(UiEvent::Resize(UiResize::new(Some(80), Some(25), 20, 30)))
        );
        assert!(matches!(host.pop_ready_event(), Some(UiEvent::Key(_))));
        assert_eq!(host.pop_ready_event(), None);
    }

    #[test]
    fn flush_pending_resize_after_resize_only_stream() {
        let mut host = UiHost::for_terminal();
        host.ingest_ui_event(UiEvent::Resize(UiResize::new(Some(80), Some(25), 80, 25)));
        assert!(host.flush_pending_resize());
        assert_eq!(
            host.pop_ready_event(),
            Some(UiEvent::Resize(UiResize::new(Some(80), Some(25), 80, 25)))
        );
        assert!(!host.flush_pending_resize());
    }

    #[test]
    fn read_next_blocking_drains_resize_burst_then_key() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();
        let mut host = UiHost::for_terminal();

        session
            .open(&mut console, &mut key_input, loc())
            .expect("open");
        key_input.push_console_event(ConsoleEvent::resize(1, 1));
        key_input.push_console_event(ConsoleEvent::resize(100, 40));
        key_input.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
            key_kind_index("Escape"),
            '\u{1b}',
            false,
            false,
            false,
            false,
        )));

        let first = host
            .read_next_blocking_terminal(&session, &mut console, &mut key_input, loc())
            .expect("resize");
        assert_eq!(
            first,
            UiEvent::Resize(UiResize::new(Some(80), Some(25), 100, 40))
        );
        let second = host
            .read_next_blocking_terminal(&session, &mut console, &mut key_input, loc())
            .expect("key");
        assert!(matches!(second, UiEvent::Key(_)));
    }

    #[test]
    fn trace_hook_runs_on_resize_and_flush() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);

        fn hook(_: &str) {
            CALLS.fetch_add(1, Ordering::Relaxed);
        }

        CALLS.store(0, Ordering::Relaxed);
        let mut host = UiHost::for_terminal();
        host.set_trace_hook(Some(hook as fn(&str)));
        host.ingest_ui_event(UiEvent::Resize(UiResize::new(Some(80), Some(25), 1, 1)));
        assert!(CALLS.load(Ordering::Relaxed) >= 1);
        let before = CALLS.load(Ordering::Relaxed);
        assert!(host.flush_pending_resize());
        assert!(CALLS.load(Ordering::Relaxed) > before);
    }

    #[test]
    fn resize_suggests_redraw_key_does_not() {
        let r = UiEvent::Resize(UiResize::new(Some(80), Some(25), 1, 1));
        assert!(r.suggests_request_redraw());
        let k = UiEvent::Key(crate::ConsoleKeyEvent::new(
            key_kind_index("A"),
            'a',
            false,
            false,
            false,
            false,
        ));
        assert!(!k.suggests_request_redraw());
    }
}
