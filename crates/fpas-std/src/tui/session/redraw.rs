//! Damage tracking and hosted paint-cycle operations.

use super::TuiSession;
use crate::DamageRegion;
use crate::console::Console;
use crate::error::StdError;
use fpas_bytecode::SourceLocation;

impl TuiSession {
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
}
