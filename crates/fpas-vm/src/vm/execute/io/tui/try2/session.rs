//! Try-2 TUI session state: view registry and owned widget roots.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-architecture.md`

use super::registry::{ViewKind, ViewRegistry};
use crate::vm::shared::{TurboVisionMenu, TurboVisionRect, TurboVisionStatusItem};
use std::collections::HashMap;
use std::collections::HashSet;
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::views::button::Button;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::window::Window;

/// Button constructed by `Button.New` and not yet attached to a parent.
pub(crate) struct DetachedButton {
    pub button: Box<Button>,
    pub local_bounds: Rect,
}

/// Static text constructed by `StaticText.New` and not yet attached to a parent.
pub(crate) struct DetachedStaticText {
    pub static_text: Box<StaticText>,
    pub local_bounds: Rect,
}

/// Menu bar data owned by try-2 until attached via `Application.SetMenuBar`.
pub(crate) struct Try2MenuBarState {
    pub bounds: TurboVisionRect,
    pub menus: Vec<TurboVisionMenu>,
}

/// Status line data owned by try-2 until attached via `Application.SetStatusLine`.
pub(crate) struct Try2StatusLineState {
    pub bounds: TurboVisionRect,
    pub items: Vec<TurboVisionStatusItem>,
}

/// Owned top-level widget waiting for modal exec or desktop attach.
pub(crate) enum Try2Root {
    ModalDialog(Box<Dialog>),
    Window(Box<Window>),
}

/// Rust-owned try-2 session (coexists with try-1 `TurboVisionState` until phase 7).
#[derive(Default)]
pub(crate) struct Try2Session {
    pub registry: ViewRegistry,
    session_open: bool,
    app_handle: Option<u32>,
    roots: HashMap<u32, Try2Root>,
    /// Buttons from `Button.New` awaiting parent attach.
    detached_buttons: HashMap<u32, DetachedButton>,
    /// Static text from `StaticText.New` awaiting parent attach.
    detached_static_texts: HashMap<u32, DetachedStaticText>,
    /// Headless test click targets keyed by try-2 button handle.
    button_clicks: HashMap<u32, Point>,
    /// Window handles attached to the upstream desktop via `Desktop.Add`.
    desktop_windows: HashSet<u32>,
    menu_bars: HashMap<u32, Try2MenuBarState>,
    status_lines: HashMap<u32, Try2StatusLineState>,
    attached_menu_bar: Option<u32>,
    attached_status_line: Option<u32>,
}

impl Try2Session {
    /// Clears registry, roots, and session flags.
    pub fn reset(&mut self) {
        self.registry.clear();
        self.session_open = false;
        self.app_handle = None;
        self.roots.clear();
        self.detached_buttons.clear();
        self.detached_static_texts.clear();
        self.button_clicks.clear();
        self.desktop_windows.clear();
        self.menu_bars.clear();
        self.status_lines.clear();
        self.attached_menu_bar = None;
        self.attached_status_line = None;
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
    pub fn insert_detached_button(&mut self, button: Box<Button>, local_bounds: Rect) -> u32 {
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

    /// Removes a detached button for parent attach.
    pub fn take_detached_button(&mut self, handle: u32) -> Option<DetachedButton> {
        self.detached_buttons.remove(&handle)
    }

    /// Inserts a detached static text and returns its FPAS handle.
    pub fn insert_detached_static_text(
        &mut self,
        static_text: Box<StaticText>,
        local_bounds: Rect,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::StaticText);
        self.detached_static_texts.insert(
            handle,
            DetachedStaticText {
                static_text,
                local_bounds,
            },
        );
        handle
    }

    /// Removes a detached static text for parent attach.
    pub fn take_detached_static_text(&mut self, handle: u32) -> Option<DetachedStaticText> {
        self.detached_static_texts.remove(&handle)
    }

    /// Stores menu bar data for a registry handle.
    pub fn insert_menu_bar(&mut self, handle: u32, state: Try2MenuBarState) {
        self.menu_bars.insert(handle, state);
    }

    /// Stores status line data for a registry handle.
    pub fn insert_status_line(&mut self, handle: u32, state: Try2StatusLineState) {
        self.status_lines.insert(handle, state);
    }

    /// Returns the attached menu bar handle, if any.
    #[must_use]
    pub fn attached_menu_bar(&self) -> Option<u32> {
        self.attached_menu_bar
    }

    /// Returns the attached status line handle, if any.
    #[must_use]
    pub fn attached_status_line(&self) -> Option<u32> {
        self.attached_status_line
    }

    /// Marks a menu bar as application chrome.
    pub fn set_attached_menu_bar(&mut self, handle: u32) {
        self.attached_menu_bar = Some(handle);
    }

    /// Marks a status line as application chrome.
    pub fn set_attached_status_line(&mut self, handle: u32) {
        self.attached_status_line = Some(handle);
    }

    /// Snapshot of the attached menu bar, if set.
    #[must_use]
    pub fn attached_menu_bar_snapshot(&self) -> Option<&Try2MenuBarState> {
        self.attached_menu_bar
            .and_then(|handle| self.menu_bars.get(&handle))
    }

    /// Snapshot of the attached status line, if set.
    #[must_use]
    pub fn attached_status_line_snapshot(&self) -> Option<&Try2StatusLineState> {
        self.attached_status_line
            .and_then(|handle| self.status_lines.get(&handle))
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

    /// Returns `true` when `handle` was passed to `Desktop.Add`.
    #[must_use]
    pub fn is_on_desktop(&self, handle: u32) -> bool {
        self.desktop_windows.contains(&handle)
    }

    /// Records that a window handle now lives on the upstream desktop.
    pub fn mark_desktop_window(&mut self, handle: u32) {
        self.desktop_windows.insert(handle);
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
