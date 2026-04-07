//! `Std.Tui` shared semantic/compiler constants.
//!
//! **Documentation:** `docs/pascal/std/tui.md` (from the repository root).

use crate::ConsoleKeyEvent;
use crate::console::{Console, KeyInput};
use crate::console_event::{ConsoleEvent, event_kind_index};
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use std::time::{Duration, Instant};

pub const TUI_EVENT_KIND_VARIANTS: &[&str] = &["Key", "Resize"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiEvent {
    Key(ConsoleKeyEvent),
    Resize { width: i64, height: i64 },
}

#[derive(Debug, Default)]
pub struct TuiSession {
    open: bool,
    redraw_pending: bool,
    owns_raw_mode: bool,
    owns_alt_screen: bool,
}

impl TuiSession {
    pub fn open(
        &mut self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if self.open {
            return Err(session_state_error(
                "Application.Open() cannot open a second Std.Tui session while one is already active.",
                "Close the current application with `Application.Close(App)` before opening a new one.",
                location,
            ));
        }

        self.open = true;
        self.redraw_pending = false;
        self.owns_raw_mode = false;
        self.owns_alt_screen = false;

        if !console.has_terminal_writer() {
            return Ok(());
        }

        key_input.enable_raw_mode_explicit(location)?;
        self.owns_raw_mode = true;

        if let Err(error) = console.enter_alt_screen(location) {
            let _ = key_input.disable_raw_mode_explicit(location);
            self.open = false;
            self.owns_raw_mode = false;
            return Err(error);
        }

        self.owns_alt_screen = true;
        Ok(())
    }

    pub fn close(
        &mut self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Application.Close() requires an open Std.Tui application session.",
            "Call `Application.Open()` before closing the application session.",
            location,
        )?;

        let mut first_error = None;

        if self.owns_alt_screen
            && let Err(error) = console.leave_alt_screen(location)
        {
            first_error = Some(error);
        }

        if self.owns_raw_mode
            && let Err(error) = key_input.disable_raw_mode_explicit(location)
            && first_error.is_none()
        {
            first_error = Some(error);
        }

        self.open = false;
        self.redraw_pending = false;
        self.owns_raw_mode = false;
        self.owns_alt_screen = false;

        if let Some(error) = first_error {
            return Err(error);
        }

        Ok(())
    }

    pub fn size(
        &self,
        console: &mut Console,
        location: SourceLocation,
    ) -> Result<(i64, i64), StdError> {
        self.ensure_open(
            "Application.Size(App) requires an open Std.Tui application session.",
            "Open the application first and keep the returned handle alive while querying its size.",
            location,
        )?;

        Ok((console.screen_width(), console.screen_height()))
    }

    pub fn read_event(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<TuiEvent, StdError> {
        self.ensure_open(
            "Application.ReadEvent(App) requires an open Std.Tui application session.",
            "Open the application before waiting for events.",
            location,
        )?;

        loop {
            let event = key_input.read_event(location)?;
            if let Some(mapped) = map_console_event(console, event) {
                return Ok(mapped);
            }
        }
    }

    pub fn read_event_timeout(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        timeout_ms: i64,
        location: SourceLocation,
    ) -> Result<Option<TuiEvent>, StdError> {
        self.ensure_open(
            "Application.ReadEventTimeout(App, Milliseconds) requires an open Std.Tui application session.",
            "Open the application before waiting for timed events.",
            location,
        )?;

        let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(0) as u64);

        loop {
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }

            let remaining = deadline
                .duration_since(now)
                .as_millis()
                .min(i64::MAX as u128) as i64;

            match key_input.read_event_timeout(remaining, location)? {
                Some(event) => {
                    if let Some(mapped) = map_console_event(console, event) {
                        return Ok(Some(mapped));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    pub fn poll_event(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<Option<TuiEvent>, StdError> {
        self.ensure_open(
            "Application.PollEvent(App) requires an open Std.Tui application session.",
            "Open the application before polling for events.",
            location,
        )?;

        loop {
            match key_input.poll_event(location)? {
                Some(event) => {
                    if let Some(mapped) = map_console_event(console, event) {
                        return Ok(Some(mapped));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    pub fn request_redraw(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.ensure_open(
            "Application.RequestRedraw(App) requires an open Std.Tui application session.",
            "Open the application before requesting a redraw.",
            location,
        )?;

        self.redraw_pending = true;
        Ok(())
    }

    pub fn take_redraw_pending(&mut self, location: SourceLocation) -> Result<bool, StdError> {
        self.ensure_open(
            "Application.RedrawPending(App) requires an open Std.Tui application session.",
            "Open the application before checking redraw state.",
            location,
        )?;

        let pending = self.redraw_pending;
        self.redraw_pending = false;
        Ok(pending)
    }

    fn ensure_open(
        &self,
        message: &'static str,
        help: &'static str,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if self.open {
            return Ok(());
        }

        Err(session_state_error(message, help, location))
    }
}

fn map_console_event(console: &mut Console, event: ConsoleEvent) -> Option<TuiEvent> {
    if event.kind == event_kind_index("Resize") {
        console.resize(event.width as u16, event.height as u16);
        return Some(TuiEvent::Resize {
            width: event.width,
            height: event.height,
        });
    }

    if event.kind == event_kind_index("Key") {
        return Some(TuiEvent::Key(event.key));
    }

    None
}

fn session_state_error(
    message: &'static str,
    help: &'static str,
    location: SourceLocation,
) -> StdError {
    std_runtime_error(RUNTIME_CONSOLE_STATE_ERROR, message, help, location)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_event::key_kind_index;
    use fpas_bytecode::SourceLocation;

    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn tui_session_open_close_reopen_succeeds_without_terminal_writer() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("first open should succeed");
        session
            .close(&mut console, &mut key_input, test_location())
            .expect("close should succeed");
        session
            .open(&mut console, &mut key_input, test_location())
            .expect("reopen should succeed");
    }

    #[test]
    fn tui_session_second_open_is_rejected() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("first open should succeed");

        let error = session
            .open(&mut console, &mut key_input, test_location())
            .expect_err("second open should fail");

        assert!(
            error
                .message
                .contains("cannot open a second Std.Tui session"),
            "unexpected error message: {}",
            error.message
        );
    }

    #[test]
    fn tui_session_request_redraw_is_consumed_once() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("open should succeed");
        session
            .request_redraw(test_location())
            .expect("request redraw should succeed");

        let first = session
            .take_redraw_pending(test_location())
            .expect("first redraw check should succeed");
        let second = session
            .take_redraw_pending(test_location())
            .expect("second redraw check should succeed");

        assert!(first);
        assert!(!second);
    }

    #[test]
    fn tui_session_size_requires_open_session() {
        let session = TuiSession::default();
        let mut console = Console::new();

        let error = session
            .size(&mut console, test_location())
            .expect_err("size without open session should fail");

        assert!(
            error
                .message
                .contains("requires an open Std.Tui application session"),
            "unexpected error message: {}",
            error.message
        );
    }

    #[test]
    fn tui_session_read_event_maps_resize_and_updates_console_size() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("open should succeed");

        key_input.push_console_event(ConsoleEvent::resize(120, 40));

        let event = session
            .read_event(&mut console, &mut key_input, test_location())
            .expect("read event should succeed");

        assert_eq!(
            event,
            TuiEvent::Resize {
                width: 120,
                height: 40
            }
        );
        assert_eq!(console.screen_width(), 120);
        assert_eq!(console.screen_height(), 40);
    }

    #[test]
    fn tui_session_poll_event_skips_unsupported_events_until_key() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("open should succeed");

        key_input.push_console_event(ConsoleEvent::focus_gained());
        key_input.push_console_event(ConsoleEvent::paste("ignored".to_string()));
        key_input.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
            key_kind_index("Space"),
            ' ',
            false,
            false,
            false,
            false,
        )));

        let event = session
            .poll_event(&mut console, &mut key_input, test_location())
            .expect("poll event should succeed")
            .expect("key event should be available");

        assert!(
            matches!(event, TuiEvent::Key(ConsoleKeyEvent { kind, .. }) if kind == key_kind_index("Space"))
        );
    }
}
