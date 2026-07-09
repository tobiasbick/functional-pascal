//! Headless mouse click simulation for Turbo Vision regression tests.
//!
//! **Documentation:** `docs/pascal/std/tui/app/testing.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::{TurboVisionObject, TurboVisionRect};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::command::CM_RADIO_SELECTED;
use turbo_vision::core::event::EventType;

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

        if self.try2_should_handle_application_run() {
            return self.try2_test_click_mouse(x, y, line);
        }

        let clicked =
            self.with_tui(|tui| apply_headless_mouse_click_at(&tui.turbo_vision.objects, x, y));

        let Some(clicked) = clicked else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("No Turbo Vision control at screen position ({x}, {y})"),
                "Click inside a check box or radio button painted by the headless desktop.",
                line,
            ));
        };

        if self.headless_tv_app.is_none() {
            self.turbo_vision_paint_headless_desktop(line, true);
        }

        let handle = clicked.handle();
        let Some(click_point) = self.turbo_vision_live_view_click_point(handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("No Turbo Vision control at screen position ({x}, {y})"),
                "Call `Application.Pump` once before `Application.TestClickMouse`.",
                line,
            ));
        };

        let mut app_slot = self.headless_tv_app.take();
        let Some(app) = app_slot.as_mut() else {
            self.headless_tv_app = app_slot;
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Headless Turbo Vision session is not initialized",
                "Call `Application.Pump` once before `Application.TestClickMouse`.",
                line,
            ));
        };

        app.push_mouse_down(click_point.x, click_point.y);
        let dispatched = app.dispatch_next_input_event().map_err(|error| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Headless Turbo Vision input dispatch failed: {error}"),
                "Retry after `Application.Pump` rebuilds the desktop.",
                line,
            )
        })?;

        self.headless_tv_app = app_slot;

        let Some(event) = dispatched else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("No Turbo Vision control at screen position ({x}, {y})"),
                "Click inside a check box or radio button painted by the headless desktop.",
                line,
            ));
        };

        if !mouse_click_consumed(&event) {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("No Turbo Vision control at screen position ({x}, {y})"),
                "Click inside a check box or radio button painted by the headless desktop.",
                line,
            ));
        }

        self.mark_turbo_vision_headless_repaint();
        Ok(())
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

enum HeadlessMouseClick {
    CheckBox(u32),
    RadioButton(u32),
}

impl HeadlessMouseClick {
    fn handle(self) -> u32 {
        match self {
            Self::CheckBox(handle) | Self::RadioButton(handle) => handle,
        }
    }
}

fn apply_headless_mouse_click_at(
    objects: &std::collections::HashMap<u32, TurboVisionObject>,
    x: i16,
    y: i16,
) -> Option<HeadlessMouseClick> {
    for object in objects.values() {
        let (parent_bounds, children) = match object {
            TurboVisionObject::Window(window) if window.on_desktop => {
                (window.bounds, &window.children)
            }
            TurboVisionObject::Dialog(dialog) => (dialog.bounds, &dialog.children),
            _ => continue,
        };
        if let Some(click) = hit_test_children(objects, parent_bounds, children, x, y) {
            return Some(click);
        }
    }
    None
}

fn hit_test_children(
    objects: &std::collections::HashMap<u32, TurboVisionObject>,
    parent_bounds: TurboVisionRect,
    children: &[u32],
    x: i16,
    y: i16,
) -> Option<HeadlessMouseClick> {
    for handle in children {
        let Some(child) = objects.get(handle) else {
            continue;
        };
        let bounds = match child {
            TurboVisionObject::CheckBox(check_box) => check_box.bounds,
            TurboVisionObject::RadioButton(radio_button) => radio_button.bounds,
            _ => continue,
        };
        if point_in_bounds(parent_bounds, bounds, x, y) {
            return match child {
                TurboVisionObject::CheckBox(_) => Some(HeadlessMouseClick::CheckBox(*handle)),
                TurboVisionObject::RadioButton(_) => Some(HeadlessMouseClick::RadioButton(*handle)),
                _ => None,
            };
        }
    }
    None
}

fn point_in_bounds(
    parent_bounds: TurboVisionRect,
    local_bounds: TurboVisionRect,
    x: i16,
    y: i16,
) -> bool {
    let left = parent_bounds.x.saturating_add(local_bounds.x);
    let top = parent_bounds.y.saturating_add(local_bounds.y);
    let right = left.saturating_add(local_bounds.width);
    let bottom = top.saturating_add(local_bounds.height);
    x >= left && x < right && y >= top && y < bottom
}

fn mouse_click_consumed(event: &turbo_vision::core::event::Event) -> bool {
    event.what == EventType::Nothing
        || (event.what == EventType::Broadcast && event.command == CM_RADIO_SELECTED)
}
