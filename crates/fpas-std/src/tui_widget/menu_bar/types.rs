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

/// CRT colors used while painting a menu bar widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuBarStyle {
    /// Normal bar background.
    pub bar_bg: u8,
    /// Normal bar foreground.
    pub bar_fg: u8,
    /// Foreground for the Alt shortcut letter in normal state (Turbo Pascal red).
    pub accel_fg: u8,
    /// Background for the hovered enabled item.
    pub highlight_bg: u8,
    /// Foreground for the hovered enabled item.
    pub highlight_fg: u8,
    /// Foreground for disabled items.
    pub disabled_fg: u8,
}

impl Default for MenuBarStyle {
    fn default() -> Self {
        Self {
            bar_bg: 7,
            bar_fg: 0,
            accel_fg: 4,
            highlight_bg: 0,
            highlight_fg: 7,
            disabled_fg: 8,
        }
    }
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

/// Highlight and shortcut paint colors for one menu label row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tui_widget) struct MenuLabelPaint {
    pub fg: u8,
    pub bg: u8,
    pub accel_fg: u8,
    pub hovered: bool,
}

/// Tracks which bar item and popup row are open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct OpenSubmenu {
    pub bar_index: usize,
    pub entry_index: usize,
}
