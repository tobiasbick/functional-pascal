//! Workspace loading for multi-project repositories.
//!
//! Documentation: `docs/pascal/10-projects.md`

mod discover;
mod loading;
mod resolve;
mod test_discover;

pub use discover::discover_run_project_in_workspace;
pub use loading::{LoadedWorkspace, discover_workspace_file, load_workspace};
pub use resolve::resolve_workspace_dependency_paths;
pub use test_discover::discover_test_projects_in_workspace;
