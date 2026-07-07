//! Try-2 TUI session state: view registry and owned widget roots.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-architecture.md`

use super::registry::{ViewKind, ViewRegistry};
use std::collections::HashMap;
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::views::button::Button;
use turbo_vision::views::dialog::Dialog;

/// Button constructed by `Button.New` and not yet attached to a dialog.
pub(crate) struct DetachedButton {
    pub button: Box<Button>,
    pub local_bounds: Rect,
}

/// Owned top-level widget waiting for modal exec or desktop attach.
pub(crate) enum Try2Root {
    ModalDialog(Box<Dialog>),
}

/// Rust-owned try-2 session (coexists with try-1 `TurboVisionState` until phase 7).
#[derive(Default)]
pub(crate) struct Try2Session {
    pub registry: ViewRegistry,
    session_open: bool,
    app_handle: Option<u32>,
    roots: HashMap<u32, Try2Root>,
    /// Buttons from `Button.New` awaiting `Dialog.Add`.
    detached_buttons: HashMap<u32, DetachedButton>,
    /// Headless test click targets keyed by try-2 button handle.
    button_clicks: HashMap<u32, Point>,
}

impl Try2Session {
    /// Clears registry, roots, and session flags.
    pub fn reset(&mut self) {
        self.registry.clear();
        self.session_open = false;
        self.app_handle = None;
        self.roots.clear();
        self.detached_buttons.clear();
        self.button_clicks.clear();
    }

    /// Returns `true` after [`Self::open`].
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.session_open
    }

    /// Opens a try-2 session and returns the application handle token.
    pub fn open(&mut self) -> u32 {
        self.reset();
        self.session_open = true;
        let handle = self.registry.allocate(0, ViewKind::Application);
        self.app_handle = Some(handle);
        handle
    }

    /// Application handle for the active session, if any.
    #[must_use]
    pub fn app_handle(&self) -> Option<u32> {
        self.app_handle
    }

    /// Inserts a root widget and returns its FPAS handle.
    pub fn insert_root(&mut self, root: Try2Root, kind: ViewKind) -> u32 {
        let handle = self.registry.allocate(0, kind);
        self.roots.insert(handle, root);
        handle
    }

    /// Mutable access to a root widget by handle.
    pub fn root_mut(&mut self, handle: u32) -> Option<&mut Try2Root> {
        self.roots.get_mut(&handle)
    }

    /// Removes a root widget (for modal exec).
    pub fn take_root(&mut self, handle: u32) -> Option<Try2Root> {
        self.roots.remove(&handle)
    }

    /// Inserts a detached button and returns its FPAS handle.
    pub fn insert_detached_button(
        &mut self,
        button: Box<Button>,
        local_bounds: Rect,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::Button);
        self.detached_buttons.insert(
            handle,
            DetachedButton {
                button,
                local_bounds,
            },
        );
        handle
    }

    /// Removes a detached button for `Dialog.Add`.
    pub fn take_detached_button(&mut self, handle: u32) -> Option<DetachedButton> {
        self.detached_buttons.remove(&handle)
    }

    /// Records the screen point used by headless `Application.TestClickButton`.
    pub fn set_button_click_point(&mut self, handle: u32, point: Point) {
        self.button_clicks.insert(handle, point);
    }

    /// Screen point for a try-2 button handle, if registered.
    #[must_use]
    pub fn button_click_point(&self, handle: u32) -> Option<Point> {
        self.button_clicks.get(&handle).copied()
    }

    /// Validates session is open.
    pub fn require_open(&self) -> bool {
        self.session_open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_allocates_application_handle() {
        let mut session = Try2Session::default();
        let h = session.open();
        assert!(session.is_open());
        assert_eq!(session.app_handle(), Some(h));
        assert_eq!(session.registry.len(), 1);
    }

    #[test]
    fn reset_clears_roots() {
        let mut session = Try2Session::default();
        session.open();
        let bounds = turbo_vision::core::geometry::Rect::new(0, 0, 10, 5);
        session.insert_root(
            Try2Root::ModalDialog(Dialog::new_modal(bounds, "T")),
            ViewKind::Dialog,
        );
        session.reset();
        assert!(!session.is_open());
        assert!(session.roots.is_empty());
    }
}
