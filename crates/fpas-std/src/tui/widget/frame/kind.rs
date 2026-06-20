//! Frame kind presets for Turbo Vision-style windows and dialogs.
//!
//! Visual palette data is added with the frame painter; this file only carries semantic kind and
//! the capabilities that are implemented today.
//!
//! Plan: `docs/future/windows-dialogs/README.md`
//! Spec: `docs/pascal/std/tui/app/README.md`

use super::FrameCapabilities;

/// High-level frame kind used by frame root creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    /// Non-modal desktop window.
    Window,
    /// Modal dialog frame.
    Dialog,
}

impl FrameKind {
    /// Default capabilities for the behavior currently implemented by the frame foundation.
    ///
    /// Scroll geometry is available for both windows and dialogs. Close, zoom, move, and resize
    /// remain disabled until their routing and host actions are implemented.
    #[must_use]
    pub const fn default_capabilities(self) -> FrameCapabilities {
        match self {
            Self::Window | Self::Dialog => FrameCapabilities::scrollable(),
        }
    }
}
