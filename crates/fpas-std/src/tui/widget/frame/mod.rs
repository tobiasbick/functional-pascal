//! Turbo Vision-style frame widget primitives.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

mod chrome;
mod geometry;
mod hit;
mod interaction;
mod kind;
mod layout;
mod root;
mod scroll;
mod state;
mod style;
mod widget;
mod window_list;

#[cfg(test)]
mod tests;

pub use geometry::{
    FrameButtonSlots, FrameCapabilities, FrameContentSize, FrameGeometry, FrameGeometryError,
    FrameScrollbars,
};
pub use hit::FrameChromeHit;
pub use kind::FrameKind;
pub use root::{FrameRoot, FrameRootSpec, FramedDialogRoot, register_framed_dialog_root};
pub use scroll::{FrameScrollHit, FrameScrollState};
pub use state::{FrameResizeEdge, FrameRootState, FrameScrollInteraction, WindowInteraction};
pub use style::FrameStyle;
pub use widget::FrameWidget;
pub use window_list::FrameWindowDescriptor;
