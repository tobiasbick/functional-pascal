//! Headless mouse click simulation for Turbo Vision regression tests.
//!
//! **Documentation:** `docs/pascal/std/tui/app/testing.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::{TurboVisionObject, TurboVisionRect};
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

        let clicked = self.with_tui(|tui| {
            let x = i16::try_from(x).ok()?;
            let y = i16::try_from(y).ok()?;
            apply_headless_mouse_click_at(&tui.turbo_vision.objects, x, y)
        });

        let Some(clicked) = clicked else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("No Turbo Vision control at screen position ({x}, {y})"),
                "Click inside a check box or radio button painted by the headless desktop.",
                line,
            ));
        };

        match clicked {
            HeadlessMouseClick::CheckBox(handle) => {
                self.with_tui(|tui| {
                    let Some(TurboVisionObject::CheckBox(check_box)) =
                        tui.turbo_vision.objects.get_mut(&handle)
                    else {
                        return Ok(());
                    };
                    check_box.checked_cell.set(!check_box.checked_cell.read());
                    Ok(())
                })?;
            }
            HeadlessMouseClick::RadioButton(handle) => {
                self.select_radio_button_group(handle, line)?;
            }
        }

        self.mark_turbo_vision_tree_dirty();
        Ok(())
    }

    fn select_radio_button_group(
        &mut self,
        handle: u32,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.with_tui(|tui| {
            let group_id = match tui.turbo_vision.objects.get(&handle) {
                Some(TurboVisionObject::RadioButton(radio_button)) => radio_button.group_id,
                _ => {
                    return Err(super::tv_geometry::unknown_handle_error(
                        "RadioButton",
                        handle,
                        line,
                    ));
                }
            };
            for object in tui.turbo_vision.objects.values_mut() {
                let TurboVisionObject::RadioButton(radio_button) = object else {
                    continue;
                };
                if radio_button.group_id == group_id {
                    radio_button.selected_cell.set(false);
                }
            }
            let Some(TurboVisionObject::RadioButton(radio_button)) =
                tui.turbo_vision.objects.get_mut(&handle)
            else {
                return Err(super::tv_geometry::unknown_handle_error(
                    "RadioButton",
                    handle,
                    line,
                ));
            };
            radio_button.selected_cell.set(true);
            Ok(())
        })
    }
}

enum HeadlessMouseClick {
    CheckBox(u32),
    RadioButton(u32),
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
