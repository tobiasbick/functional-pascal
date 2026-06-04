//! Workspace loading for multi-project repositories.
//!
//! Documentation: `docs/pascal/10-projects.md`

mod loading;

pub use loading::{LoadedWorkspace, discover_workspace_file, load_workspace};
