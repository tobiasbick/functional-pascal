//! Recoverable project and workspace context loading.

mod context;
mod discovery;
mod project;

pub use context::{WorkspaceContext, WorkspaceIssue, WorkspaceKind};
pub use project::ProjectContext;
