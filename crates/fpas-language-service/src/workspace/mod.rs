//! Recoverable project and workspace context loading.

mod context;
mod discovery;
mod project;
mod standard_library;

pub use context::{WorkspaceContext, WorkspaceIssue, WorkspaceKind};
pub use project::ProjectContext;
pub(crate) use standard_library::StandardLibraryContext;
