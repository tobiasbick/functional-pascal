//! CRT palettes for painted window and dialog frames.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use super::FrameKind;

/// Active, inactive, and client colors used by a [`super::FrameWidget`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameStyle {
    /// Active title and border background.
    pub active_bg: u8,
    /// Active title and border foreground.
    pub active_fg: u8,
    /// Inactive title and border background.
    pub inactive_bg: u8,
    /// Inactive title and border foreground.
    pub inactive_fg: u8,
    /// Client-area background.
    pub client_bg: u8,
    /// Client-area foreground.
    pub client_fg: u8,
}

impl FrameStyle {
    /// Return the built-in Turbo Vision-style palette for one frame kind.
    #[must_use]
    pub const fn for_kind(kind: FrameKind) -> Self {
        match kind {
            FrameKind::Window => Self {
                active_bg: 9,
                active_fg: 15,
                inactive_bg: 1,
                inactive_fg: 7,
                client_bg: 7,
                client_fg: 0,
            },
            FrameKind::Dialog => Self {
                active_bg: 7,
                active_fg: 0,
                inactive_bg: 7,
                inactive_fg: 0,
                client_bg: 7,
                client_fg: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_style_palettes_keep_window_and_dialog_focus_contract() {
        let window = FrameStyle::for_kind(FrameKind::Window);
        assert_eq!((window.active_fg, window.active_bg), (15, 9));
        assert_eq!((window.inactive_fg, window.inactive_bg), (7, 1));
        assert_eq!((window.client_fg, window.client_bg), (0, 7));
        assert_ne!(
            (window.active_fg, window.active_bg),
            (window.inactive_fg, window.inactive_bg)
        );

        let dialog = FrameStyle::for_kind(FrameKind::Dialog);
        assert_eq!((dialog.active_fg, dialog.active_bg), (0, 7));
        assert_eq!((dialog.inactive_fg, dialog.inactive_bg), (0, 7));
        assert_eq!((dialog.client_fg, dialog.client_bg), (0, 7));
    }
}
