//! Headless input and test-result session state.

use super::*;

impl TurboVisionSession {
    pub fn set_button_click_point(&mut self, handle: u32, point: Point) {
        self.button_clicks.insert(handle, point);
    }

    /// Screen point for a Turbo Vision button handle, if registered.
    #[must_use]
    pub fn button_click_point(&self, handle: u32) -> Option<Point> {
        self.button_clicks.get(&handle).copied()
    }

    /// Registers a screen-space control target for headless `TestClickMouse`.
    pub fn register_mouse_hit_target(&mut self, handle: u32, hit: Rect, click: Point) {
        self.mouse_hit_targets
            .push(MouseHitTarget { handle, hit, click });
    }

    /// Resolves a queued screen click to a desktop mouse coordinate when possible.
    #[must_use]
    pub fn mouse_hit_target_for_screen(&self, x: i16, y: i16) -> Option<MouseHitTarget> {
        for target in &self.mouse_hit_targets {
            if super::super::view_click::point_in_screen_bounds(target.hit, x, y) {
                return Some(*target);
            }
        }
        None
    }

    /// Queues a stateful control transition for headless `TestClickMouse`.
    pub fn queue_mouse_state_toggle(&mut self, handle: u32) {
        self.pending_mouse_state_toggles.push(handle);
    }

    /// Takes all stateful control transitions queued by headless mouse tests.
    pub fn take_pending_mouse_state_toggles(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.pending_mouse_state_toggles)
    }

    /// Returns `true` when `handle` was passed to `Desktop.Add`.
    #[must_use]
    pub fn is_on_desktop(&self, handle: u32) -> bool {
        self.desktop_windows.contains(&handle)
    }

    /// Records that a window handle now lives on the upstream desktop.
    pub fn mark_desktop_window(&mut self, handle: u32) {
        self.desktop_windows.insert(handle);
    }

    /// Queues the result consumed by the next headless `Application.RunFileDialog`.
    pub fn set_file_dialog_result(&mut self, result: Option<String>) {
        self.file_dialog_result = Some(result);
    }

    /// Consumes the queued headless file dialog result, if one was set.
    #[must_use]
    pub fn take_file_dialog_result(&mut self) -> Option<Option<String>> {
        self.file_dialog_result.take()
    }

    /// Queues the closing command consumed by the next headless `Application.MessageBox`.
    pub fn set_dialog_result(&mut self, command: i64) {
        self.dialog_result = Some(command);
    }

    /// Consumes the queued headless modal result, if one was set.
    #[must_use]
    pub fn take_dialog_result(&mut self) -> Option<i64> {
        self.dialog_result.take()
    }
}
