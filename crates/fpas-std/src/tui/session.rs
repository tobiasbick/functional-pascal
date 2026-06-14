use super::damage::DamageTracker;
use super::event::{TuiEvent, map_console_event, map_console_ui_event};
use crate::DamageRegion;
use crate::UiEvent;
use crate::console::{Console, KeyInput};
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use std::time::{Duration, Instant};

/// Runtime state for one hosted `Std.Tui` application session.
#[derive(Debug, Default)]
pub struct TuiSession {
    open: bool,
    damage: DamageTracker,
    redraw_hint: Option<DamageRegion>,
    owns_raw_mode: bool,
    owns_alt_screen: bool,
    owns_mouse: bool,
    /// When true, the session was opened with [`TuiSession::open_for_test`] (no terminal I/O).
    headless: bool,
}

impl TuiSession {
    /// Open a TUI application session and acquire terminal state when a writer is available.
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
        self.headless = false;
        self.damage.clear();
        self.redraw_hint = None;
        self.owns_raw_mode = false;
        self.owns_alt_screen = false;
        self.owns_mouse = false;
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

        if let Err(error) = console.enable_mouse(location) {
            let _ = console.leave_alt_screen(location);
            let _ = key_input.disable_raw_mode_explicit(location);
            self.open = false;
            self.owns_raw_mode = false;
            self.owns_alt_screen = false;
            return Err(error);
        }
        self.owns_mouse = true;
        Ok(())
    }

    /// Close the active TUI application session and restore terminal state.
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

        if self.owns_mouse
            && let Err(error) = console.disable_mouse(location)
            && first_error.is_none()
        {
            first_error = Some(error);
        }

        self.open = false;
        self.damage.clear();
        self.redraw_hint = None;
        self.owns_raw_mode = false;
        self.owns_alt_screen = false;
        self.owns_mouse = false;
        self.headless = false;

        if let Some(error) = first_error {
            return Err(error);
        }

        Ok(())
    }

    /// Open a headless TUI session for native FPAS tests (`Application.OpenForTest`).
    ///
    /// Does not acquire raw mode, alternate screen, or mouse capture. Resize the logical
    /// console to the desired virtual terminal size before calling this method.
    pub fn open_for_test(
        &mut self,
        console: &mut Console,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if self.open {
            return Err(session_state_error(
                "Application.OpenForTest() cannot open a second Std.Tui session while one is already active.",
                "Close the current application with `Application.CloseForTest(App)` before opening a new one.",
                location,
            ));
        }

        console.abort_tui_paint();
        self.open = true;
        self.headless = true;
        self.damage.clear();
        self.redraw_hint = None;
        self.owns_raw_mode = false;
        self.owns_alt_screen = false;
        self.owns_mouse = false;
        Ok(())
    }

    /// Returns whether this session was opened headlessly for native tests.
    #[must_use]
    pub fn is_headless(&self) -> bool {
        self.headless
    }

    /// Return the current terminal size for the active application session.
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

    /// Block until the session yields a supported TUI event.
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

    /// Block until the session yields a supported hosted UI event.
    #[doc(hidden)]
    pub fn read_ui_event(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<UiEvent, StdError> {
        self.ensure_open(
            "Application.ReadEvent(App) requires an open Std.Tui application session.",
            "Open the application before waiting for events.",
            location,
        )?;

        loop {
            let event = key_input.read_event(location)?;
            if let Some(mapped) = map_console_ui_event(console, event) {
                return Ok(mapped);
            }
        }
    }

    /// Wait up to `timeout_ms` for a supported TUI event.
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

    /// Wait up to `timeout_ms` for a supported hosted UI event.
    #[doc(hidden)]
    pub fn read_ui_event_timeout(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        timeout_ms: i64,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
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
                    if let Some(mapped) = map_console_ui_event(console, event) {
                        return Ok(Some(mapped));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    /// Poll once for a supported TUI event, skipping paste and focus dispatch-only events.
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
                        match mapped {
                            TuiEvent::Paste(_) | TuiEvent::FocusGained | TuiEvent::FocusLost => {
                                continue;
                            }
                            _ => return Ok(Some(mapped)),
                        }
                    }
                }
                None => return Ok(None),
            }
        }
    }

    /// Poll once for a supported hosted UI event, skipping paste and focus dispatch events.
    #[doc(hidden)]
    pub fn poll_ui_event(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        self.ensure_open(
            "Application.PollEvent(App) requires an open Std.Tui application session.",
            "Open the application before polling for events.",
            location,
        )?;

        loop {
            match key_input.poll_event(location)? {
                Some(event) => {
                    if let Some(mapped) = map_console_ui_event(console, event) {
                        match mapped {
                            UiEvent::Paste(_) | UiEvent::FocusGained | UiEvent::FocusLost => {
                                continue;
                            }
                            _ => return Ok(Some(mapped)),
                        }
                    }
                }
                None => return Ok(None),
            }
        }
    }

    /// Poll once for a supported hosted UI event, including paste and focus dispatch events.
    #[doc(hidden)]
    pub fn poll_ui_event_all(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        self.ensure_open(
            "Application.PollEvent(App) requires an open Std.Tui application session.",
            "Open the application before polling for events.",
            location,
        )?;

        loop {
            match key_input.poll_event(location)? {
                Some(event) => {
                    if let Some(mapped) = map_console_ui_event(console, event) {
                        return Ok(Some(mapped));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    /// Mark a redraw as pending for the next hosted paint cycle.
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

    /// Consume the pending redraw flag and report whether redraw work was queued.
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

fn session_state_error(
    message: &'static str,
    help: &'static str,
    location: SourceLocation,
) -> StdError {
    std_runtime_error(RUNTIME_CONSOLE_STATE_ERROR, message, help, location)
}
