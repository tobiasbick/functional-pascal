//! Shared scrollbar colors for legacy frame chrome.

/// CRT colors for scroll bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollBarStyle {
    /// Background color.
    pub bg: u8,
    /// Track foreground color.
    pub fg: u8,
    /// Thumb foreground color.
    pub thumb_fg: u8,
    /// Arrow foreground color.
    pub arrow_fg: u8,
}
impl Default for ScrollBarStyle {
    fn default() -> Self {
        Self {
            bg: 7,
            fg: 8,
            thumb_fg: 0,
            arrow_fg: 0,
        }
    }
}
