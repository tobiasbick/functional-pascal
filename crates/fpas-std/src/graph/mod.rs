//! `Std.Graph` runtime scaffolding.
//!
//! **Documentation:** `docs/pascal/std/graph.md` (from the repository root).

mod backbuffer;
mod backend;
mod circle;
mod color;
mod event;
mod framebuffer;
mod graph_host;
mod line;
mod rect;
mod session;
mod text;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use backend::last_headless_graph_frame_for_tests;
#[doc(hidden)]
#[cfg(test)]
pub use backend::set_headless_graph_surface_size_for_tests;
#[doc(hidden)]
pub use backend::with_headless_graph_backend_for_tests;
pub use event::{
    GRAPH_EVENT_KIND_VARIANTS, GRAPH_EXIT_REASON_VARIANTS, GraphEvent, GraphEventKind,
};
pub use framebuffer::UploadedFrame;
pub use graph_host::GraphHost;
pub use session::GraphSession;
