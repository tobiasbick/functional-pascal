//! Recoverable project and workspace context loading.

mod catalog;
mod context;
mod discovery;
pub(crate) mod path_containment;
mod project;
mod standard_library;

pub use context::{WorkspaceContext, WorkspaceIssue, WorkspaceKind};
pub use project::ProjectContext;
pub(crate) use standard_library::StandardLibraryContext;
