//! Recoverable project and workspace context loading.

mod catalog;
mod context;
mod discovery;
mod issue;
pub(crate) mod path_containment;
mod project;
mod standard_library;

pub use context::{WorkspaceContext, WorkspaceKind};
pub use issue::WorkspaceIssue;
pub use project::ProjectContext;
pub(crate) use standard_library::StandardLibraryContext;
