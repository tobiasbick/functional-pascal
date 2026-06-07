//! Menu bar model types and event results.
//!
//! Spec: `docs/pascal/std/tui-app.md`

use super::super::menu_popup::MenuPopupItem;

/// One declarative menu entry supplied from Pascal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBarItem {
    /// Visible label text.
    pub label: String,
    /// Alt+letter shortcut (single character, case-insensitive). Empty means none.
    pub shortcut: String,
    /// When false, the entry is drawn disabled and ignores clicks.
    pub enabled: bool,
    /// Command dispatched through `OnCommand` on click, or `-1` when not clickable.
    pub command_id: i64,
    /// Pull-down entries shown when this top-level item is activated.
    pub submenu: Vec<MenuPopupItem>,
}

/// Result of routing one mouse or keyboard event to a menu bar widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarMouseResult {
    /// The widget did not consume the event.
    Ignored,
    /// Hover or submenu state changed; caller should redraw affected regions.
    HoverChanged,
    /// A clickable item was activated.
    Command(crate::CommandId),
}

/// Tracks which bar item and popup row are open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct OpenSubmenu {
    pub bar_index: usize,
    pub entry_index: usize,
}
