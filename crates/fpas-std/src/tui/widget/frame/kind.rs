//! Frame kind presets for Turbo Vision-style windows and dialogs.
//!
//! Plan: `docs/future/tui/completed.md`
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
    /// Default capabilities for a newly created frame.
    #[must_use]
    pub const fn default_capabilities(self) -> FrameCapabilities {
        match self {
            Self::Window | Self::Dialog => FrameCapabilities::scrollable(),
        }
    }
}
