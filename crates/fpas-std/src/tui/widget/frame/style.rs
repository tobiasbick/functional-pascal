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
