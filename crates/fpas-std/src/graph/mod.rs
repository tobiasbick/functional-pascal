//! `Std.Graph` runtime scaffolding.
//!
//! **Documentation:** `docs/future/std.graph/01-mvp.md`, `docs/future/std.graph/02-pascal-surface.md` (from the repository root).

mod event;
mod framebuffer;
mod session;
mod stub;

#[cfg(test)]
mod tests;

pub use event::{GRAPH_EVENT_KIND_VARIANTS, GraphEvent, GraphEventKind};
pub use framebuffer::UploadedFrame;
pub use session::GraphSession;
pub use stub::run_graph_intrinsic;