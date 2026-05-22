//! `Std.Graph` runtime scaffolding.
//!
//! **Documentation:** `docs/future/std.graph/01-mvp.md`, `docs/future/std.graph/02-pascal-surface.md` (from the repository root).

mod backbuffer;
mod backend;
mod color;
mod event;
mod framebuffer;
mod session;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use backend::last_headless_graph_frame_for_tests;
#[doc(hidden)]
pub use backend::with_headless_graph_backend_for_tests;
pub use event::{GRAPH_EVENT_KIND_VARIANTS, GraphEvent, GraphEventKind};
pub use framebuffer::UploadedFrame;
pub use session::GraphSession;
