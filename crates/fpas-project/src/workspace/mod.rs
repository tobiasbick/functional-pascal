//! Workspace loading for multi-project repositories.
//!
//! Documentation: `docs/pascal/10-projects.md`

mod loading;
mod resolve;

pub use loading::{LoadedWorkspace, discover_workspace_file, load_workspace};
pub use resolve::resolve_workspace_dependency_paths;
