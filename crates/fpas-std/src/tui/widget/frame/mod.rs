//! Turbo Vision-style frame widget primitives.
//!
//! This module starts with geometry only; painting and host creation are layered on top once the
//! frame rectangles are stable and covered by tests.
//!
//! Plan: `docs/future/windows-dialogs/README.md`
//! Spec: `docs/pascal/std/tui/app/README.md`

mod geometry;
mod hit;
mod interaction;
mod kind;
mod layout;
mod root;
mod state;

#[cfg(test)]
mod tests;

pub use geometry::{
    FrameButtonSlots, FrameCapabilities, FrameContentSize, FrameGeometry, FrameGeometryError,
    FrameScrollbars,
};
pub use hit::FrameChromeHit;
pub use kind::FrameKind;
pub use root::{FrameRoot, FrameRootSpec, FramedDialogRoot, register_framed_dialog_root};
pub use state::{FrameResizeEdge, FrameRootState, WindowInteraction};
