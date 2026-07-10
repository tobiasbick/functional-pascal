//! Headless mouse click simulation for Turbo Vision regression tests.
//!
//! **Documentation:** `docs/pascal/std/tui/app/testing.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

impl Worker {
    /// Queue a left mouse down at screen coordinates for headless `OpenForTest` runs.
    pub(super) fn turbo_vision_test_click_mouse(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let y = self.pop_int(line)?;
        let x = self.pop_int(line)?;
        self.pop_tui_application(line)?;

        if !self.with_tui(|tui| tui.session.is_headless()) {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.TestClickMouse is only supported in headless `Application.OpenForTest` runs",
                "Call `Application.OpenForTest` before `Application.TestClickMouse`.",
                line,
            ));
        }

        let x = i16::try_from(x).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Mouse X coordinate {x} is out of range for Turbo Vision"),
                "Use screen coordinates within the headless terminal size.",
                line,
            )
        })?;
        let y = i16::try_from(y).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Mouse Y coordinate {y} is out of range for Turbo Vision"),
                "Use screen coordinates within the headless terminal size.",
                line,
            )
        })?;

        self.try2_test_click_mouse(x, y, line)
    }

    fn try2_test_click_mouse(
        &mut self,
        x: i16,
        y: i16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let width = self.with_console(|console| console.screen_width() as u16);
        let height = self.with_console(|console| console.screen_height() as u16);
        if x < 0 || y < 0 || x >= width as i16 || y >= height as i16 {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Mouse coordinate ({x}, {y}) is outside the headless terminal"),
                "Use screen coordinates within the headless terminal size.",
                line,
            ));
        }

        self.turbo_vision_ensure_headless_app(width, height)
            .map_err(|error| {
                runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    format!("Headless Turbo Vision initialization failed: {error}"),
                    "Call `Application.OpenForTest` before `Application.TestClickMouse`.",
                    line,
                )
            })?;
        let point = self
            .try2
            .mouse_point_for_screen(x, y)
            .unwrap_or_else(|| turbo_vision::core::geometry::Point::new(x, y));

        let mut app = self.headless_tv_app.take().ok_or_else(|| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Headless Turbo Vision session is not initialized",
                "Call `Application.OpenForTest` before `Application.TestClickMouse`.",
                line,
            )
        })?;
        app.push_mouse_down(point.x, point.y);
        if app.desktop_mut().child_count() > 0 {
            let _ = app.dispatch_next_input_event();
        }
        app.push_mouse_up(point.x, point.y);
        if app.desktop_mut().child_count() > 0 {
            let _ = app.dispatch_next_input_event();
        }
        self.headless_tv_app = Some(app);
        Ok(())
    }
}
