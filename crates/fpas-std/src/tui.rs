//! `Std.Tui` shared semantic/compiler constants.
//!
//! **Documentation:** `docs/pascal/std/tui.md` (from the repository root).

use crate::ConsoleKeyEvent;
use crate::DamageRegion;
use crate::console::{Console, KeyInput};
use crate::console_event::{ConsoleEvent, event_kind_index};
use crate::error::{StdError, std_runtime_error};
use crate::tui_damage::DamageTracker;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use std::time::{Duration, Instant};

pub const TUI_EVENT_KIND_VARIANTS: &[&str] = &["Key", "Resize", "Mouse"];

/// Variants for `Std.Tui.ExitReason` (dispatch `OnExit` / future `Application.Run`); see `docs/pascal/std/tui-app.md`.
pub const TUI_EXIT_REASON_VARIANTS: &[&str] =
    &["UserQuit", "HostStop", "HostAndUserStop", "HostShutdown"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiEvent {
    Key(ConsoleKeyEvent),
    Resize {
        old_width: i64,
        old_height: i64,
        width: i64,
        height: i64,
    },
    Mouse(ConsoleEvent),
    /// Bracketed-paste content; best-effort on terminals that support it.
    Paste(ConsoleEvent),
    /// Terminal focus gained; best-effort / optional on many terminals.
    FocusGained(ConsoleEvent),
    /// Terminal focus lost; best-effort / optional on many terminals.
    FocusLost(ConsoleEvent),
}

#[derive(Debug, Default)]
pub struct TuiSession {
    open: bool,
    damage: DamageTracker,
    redraw_hint: Option<DamageRegion>,
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
        self.damage.clear();
        self.redraw_hint = None;
        self.owns_raw_mode = false;
        self.owns_alt_screen = false;
        console.abort_tui_paint();

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
        console.abort_tui_paint();

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
        self.damage.clear();
        self.redraw_hint = None;
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
                        // Paste and focus events are dispatch-only; skip them in the poll path.
                        match mapped {
                            TuiEvent::Paste(_)
                            | TuiEvent::FocusGained(_)
                            | TuiEvent::FocusLost(_) => continue,
                            _ => return Ok(Some(mapped)),
                        }
                    }
                }
                None => return Ok(None),
            }
        }
    }

    /// Like [`poll_event`](Self::poll_event) but also returns paste and focus events.
    ///
    /// Used by the hosted run loop which dispatches those events to registered handlers.
    pub fn poll_event_all(
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

        match self.redraw_hint.unwrap_or(DamageRegion::FullFrame) {
            DamageRegion::FullFrame => self.damage.mark_full(),
            DamageRegion::Rect(rect) => self.damage.mark_rect(rect),
        }
        Ok(())
    }

    /// Marks the whole application surface dirty only when no redraw damage is pending yet.
    ///
    /// This lets the hosted startup path request an initial paint without overwriting more
    /// specific dirty-rectangle information that may already have been accumulated.
    pub fn request_redraw_if_absent(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.ensure_open(
            "Application.RequestRedraw(App) requires an open Std.Tui application session.",
            "Open the application before requesting a redraw.",
            location,
        )?;

        if !self.damage.has_damage() {
            match self.redraw_hint.unwrap_or(DamageRegion::FullFrame) {
                DamageRegion::FullFrame => self.damage.mark_full(),
                DamageRegion::Rect(rect) => self.damage.mark_rect(rect),
            }
        }
        Ok(())
    }

    /// Marks a rectangular region dirty for the next hosted paint.
    ///
    /// This is currently a Rust-host detail used while Phase 7 performance work moves from
    /// whole-frame redraw requests toward partial invalidation. FPAS still observes the same
    /// application-global `OnPaint` contract.
    pub fn request_redraw_rect(
        &mut self,
        rect: crate::ViewRect,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Application.RequestRedraw(App) requires an open Std.Tui application session.",
            "Open the application before requesting a redraw.",
            location,
        )?;

        self.damage.mark_rect(rect);
        Ok(())
    }

    /// Marks the union of the old and new surface bounds dirty after a terminal resize.
    ///
    /// The public FPAS contract still treats resize as an application-global `OnPaint`, but
    /// the Rust host can track a tighter redraw rectangle while Phase 7 moves away from
    /// blanket full-frame invalidation.
    pub fn request_resize_redraw(
        &mut self,
        old_width: i64,
        old_height: i64,
        new_width: i64,
        new_height: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Application.RequestRedraw(App) requires an open Std.Tui application session.",
            "Open the application before requesting a redraw.",
            location,
        )?;

        self.damage.mark_rect(crate::ViewRect {
            x: 0,
            y: 0,
            width: old_width.max(new_width),
            height: old_height.max(new_height),
        });
        Ok(())
    }

    /// Sets a host-side redraw hint for the next explicit redraw request.
    ///
    /// Hosted input dispatch uses this to narrow `Application.RequestRedraw(App)` to a more
    /// specific dirty region when the host can associate the event with a known view.
    /// The public FPAS paint contract remains application-global.
    pub fn set_host_redraw_hint(&mut self, damage: DamageRegion) {
        self.redraw_hint = Some(damage);
    }

    /// Clears any previously installed host-side redraw hint.
    ///
    /// Hosted dispatch calls this after each input handler so redraw narrowing does not leak
    /// across unrelated events.
    pub fn clear_host_redraw_hint(&mut self) {
        self.redraw_hint = None;
    }

    /// Begins a hosted `OnPaint` frame on the console back buffer.
    ///
    /// While the frame is active, CRT-style console operations update the buffered screen state
    /// but do not flush to the terminal. The host completes the frame with
    /// [`finish_hosted_paint`](Self::finish_hosted_paint).
    pub fn begin_hosted_paint(
        &self,
        console: &mut Console,
        damage: DamageRegion,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Application.Run(App) requires an open Std.Tui application session.",
            "Open the application before the hosted run loop starts painting.",
            location,
        )?;

        console.begin_tui_paint(damage);
        Ok(())
    }

    /// Finishes a hosted `OnPaint` frame and presents the accumulated back buffer once.
    ///
    /// The Rust host may restrict terminal diff/flush work to the tracked dirty region and the
    /// console mutations recorded during that frame, while the FPAS paint contract remains a full
    /// logical frame.
    pub fn finish_hosted_paint(
        &self,
        console: &mut Console,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Application.Run(App) requires an open Std.Tui application session.",
            "Open the application before the hosted run loop starts painting.",
            location,
        )?;

        console.finish_tui_paint(location)
    }

    /// Aborts the current hosted `OnPaint` frame without presenting it.
    ///
    /// Used by the Rust host when a paint handler fails so deferred terminal output does not leak
    /// past the failing callback.
    pub fn abort_hosted_paint(&self, console: &mut Console) {
        console.abort_tui_paint();
    }

    /// Returns the pending redraw damage without clearing it.
    ///
    /// Used by the Rust host to decide whether `OnPaint` should run and which redraw scope
    /// was requested. FPAS still treats redraw as an application-global paint request.
    pub fn peek_redraw_damage(
        &self,
        location: SourceLocation,
    ) -> Result<Option<DamageRegion>, StdError> {
        self.ensure_open(
            "Application.IsRedrawPending(App) requires an open Std.Tui application session.",
            "Open the application before querying redraw state.",
            location,
        )?;

        Ok(self.damage.peek())
    }

    /// Consumes and returns the pending redraw damage region.
    ///
    /// Used by the Rust host immediately before `OnPaint` dispatch. Returning the region now
    /// keeps the host on the damage-tracking path even while the public paint contract remains
    /// full-frame.
    pub fn take_redraw_damage(
        &mut self,
        location: SourceLocation,
    ) -> Result<Option<DamageRegion>, StdError> {
        self.ensure_open(
            "Application.RedrawPending(App) requires an open Std.Tui application session.",
            "Open the application before checking redraw state.",
            location,
        )?;

        Ok(self.damage.take())
    }

    pub fn take_redraw_pending(&mut self, location: SourceLocation) -> Result<bool, StdError> {
        Ok(self.take_redraw_damage(location)?.is_some())
    }

    /// Returns whether a redraw was requested and **does not** clear the flag (peek).
    ///
    /// Used by the VM host when deciding whether to run `OnPaint` without consuming the
    /// pending state when no paint handler is registered.
    pub fn is_redraw_pending(&self, location: SourceLocation) -> Result<bool, StdError> {
        self.ensure_open(
            "Application.IsRedrawPending(App) requires an open Std.Tui application session.",
            "Open the application before querying redraw state.",
            location,
        )?;

        Ok(self.damage.has_damage())
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
        let (Ok(width), Ok(height)) = (u16::try_from(event.width), u16::try_from(event.height))
        else {
            return None;
        };
        if width == 0 || height == 0 {
            return None;
        }

        let old_width = console.screen_width();
        let old_height = console.screen_height();
        console.resize(width, height);
        return Some(TuiEvent::Resize {
            old_width,
            old_height,
            width: event.width,
            height: event.height,
        });
    }

    if event.kind == event_kind_index("Key") {
        return Some(TuiEvent::Key(event.key));
    }

    if event.kind == event_kind_index("Mouse") {
        return Some(TuiEvent::Mouse(event));
    }

    if event.kind == event_kind_index("Paste") {
        return Some(TuiEvent::Paste(event));
    }

    if event.kind == event_kind_index("FocusGained") {
        return Some(TuiEvent::FocusGained(event));
    }

    if event.kind == event_kind_index("FocusLost") {
        return Some(TuiEvent::FocusLost(event));
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
    #![allow(clippy::expect_used)]

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
    fn tui_session_is_redraw_pending_peeks_without_clearing() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("open");
        session
            .request_redraw(test_location())
            .expect("request redraw");

        assert!(
            session
                .is_redraw_pending(test_location())
                .expect("peek redraw")
        );
        assert!(
            session
                .is_redraw_pending(test_location())
                .expect("peek again")
        );
        assert_eq!(
            session
                .peek_redraw_damage(test_location())
                .expect("peek damage"),
            Some(DamageRegion::FullFrame)
        );

        let taken = session.take_redraw_pending(test_location()).expect("take");
        assert!(taken);
        assert!(
            !session
                .is_redraw_pending(test_location())
                .expect("peek after take")
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
    fn tui_session_take_redraw_damage_returns_full_frame_once() {
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
            .take_redraw_damage(test_location())
            .expect("first damage take should succeed");
        let second = session
            .take_redraw_damage(test_location())
            .expect("second damage take should succeed");

        assert_eq!(first, Some(DamageRegion::FullFrame));
        assert_eq!(second, None);
    }

    #[test]
    fn tui_session_request_redraw_rect_marks_rect_damage() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("open should succeed");
        session
            .request_redraw_rect(
                crate::ViewRect {
                    x: 3,
                    y: 4,
                    width: 5,
                    height: 6,
                },
                test_location(),
            )
            .expect("rect redraw should succeed");

        assert_eq!(
            session
                .peek_redraw_damage(test_location())
                .expect("peek damage"),
            Some(DamageRegion::Rect(crate::ViewRect {
                x: 3,
                y: 4,
                width: 5,
                height: 6,
            }))
        );
    }

    #[test]
    fn tui_session_request_redraw_rect_merges_rectangles() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("open should succeed");
        session
            .request_redraw_rect(
                crate::ViewRect {
                    x: 2,
                    y: 2,
                    width: 4,
                    height: 3,
                },
                test_location(),
            )
            .expect("first rect redraw should succeed");
        session
            .request_redraw_rect(
                crate::ViewRect {
                    x: 8,
                    y: 1,
                    width: 2,
                    height: 5,
                },
                test_location(),
            )
            .expect("second rect redraw should succeed");

        assert_eq!(
            session
                .take_redraw_damage(test_location())
                .expect("take damage"),
            Some(DamageRegion::Rect(crate::ViewRect {
                x: 2,
                y: 1,
                width: 8,
                height: 5,
            }))
        );
    }

    #[test]
    fn tui_session_request_resize_redraw_marks_union_of_old_and_new_bounds() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("open should succeed");
        session
            .request_resize_redraw(80, 25, 40, 10, test_location())
            .expect("resize redraw should succeed");

        assert_eq!(
            session
                .take_redraw_damage(test_location())
                .expect("take damage"),
            Some(DamageRegion::Rect(crate::ViewRect {
                x: 0,
                y: 0,
                width: 80,
                height: 25,
            }))
        );
    }

    #[test]
    fn tui_session_request_redraw_if_absent_marks_full_frame_when_idle() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("open should succeed");
        session
            .request_redraw_if_absent(test_location())
            .expect("conditional redraw should succeed");

        assert_eq!(
            session
                .peek_redraw_damage(test_location())
                .expect("peek damage"),
            Some(DamageRegion::FullFrame)
        );
    }

    #[test]
    fn tui_session_request_redraw_if_absent_preserves_existing_rect_damage() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("open should succeed");
        session
            .request_redraw_rect(
                crate::ViewRect {
                    x: 6,
                    y: 7,
                    width: 8,
                    height: 9,
                },
                test_location(),
            )
            .expect("rect redraw should succeed");
        session
            .request_redraw_if_absent(test_location())
            .expect("conditional redraw should succeed");

        assert_eq!(
            session
                .peek_redraw_damage(test_location())
                .expect("peek damage"),
            Some(DamageRegion::Rect(crate::ViewRect {
                x: 6,
                y: 7,
                width: 8,
                height: 9,
            }))
        );
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
                old_width: 80,
                old_height: 25,
                width: 120,
                height: 40
            }
        );
        assert_eq!(console.screen_width(), 120);
        assert_eq!(console.screen_height(), 40);
    }

    #[test]
    fn tui_session_ignores_invalid_resize_events() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();

        session
            .open(&mut console, &mut key_input, test_location())
            .expect("open should succeed");

        key_input.push_console_event(ConsoleEvent::resize(0, 40));
        key_input.push_console_event(ConsoleEvent::resize(-1, 40));
        key_input.push_console_event(ConsoleEvent::resize(i64::from(u16::MAX) + 1, 40));
        key_input.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
            key_kind_index("Enter"),
            '\n',
            false,
            false,
            false,
            false,
        )));

        let event = session
            .read_event(&mut console, &mut key_input, test_location())
            .expect("read event should skip invalid resize events");

        assert!(matches!(event, TuiEvent::Key(_)));
        assert_eq!(console.screen_width(), 80);
        assert_eq!(console.screen_height(), 25);
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
