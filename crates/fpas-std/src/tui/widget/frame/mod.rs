//! Turbo Vision-style frame widget primitives.
//!
//! This module starts with geometry only; painting and host creation are layered on top once the
//! frame rectangles are stable and covered by tests.
//!
//! Plan: `docs/future/windows-dialogs/README.md`
//! Spec: `docs/pascal/std/tui/app/README.md`

mod geometry;
mod kind;
mod root;

pub use geometry::{
    FrameButtonSlots, FrameCapabilities, FrameContentSize, FrameGeometry, FrameGeometryError,
    FrameScrollbars,
};
pub use kind::FrameKind;
pub use root::{FrameRoot, FrameRootSpec};
