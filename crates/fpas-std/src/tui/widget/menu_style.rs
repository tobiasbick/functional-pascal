//! Shared CRT color styles for menu bar and pull-down popup painting.
//!
//! Spec: `docs/pascal/std/tui/app.md`

/// CRT colors used while painting menu bar and popup widgets.
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

/// Highlight and shortcut paint colors for one menu label row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tui::widget) struct MenuLabelPaint {
    pub fg: u8,
    pub bg: u8,
    pub accel_fg: u8,
    pub hovered: bool,
}
