//! Project loading and user-unit linking for Functional Pascal.
//!
//! Documentation:
//! - `docs/pascal/09-units.md`
//! - `docs/pascal/10-projects.md`

mod common;
mod dependencies;
mod link;
mod loading;
mod model;
mod paths;
mod test_sources;
mod workspace;

pub use link::{
    LinkedProgram, build_library_check_with_source_map, build_program,
    build_program_with_source_map,
};
pub use loading::load_project;
pub use model::{LibraryExportPolicy, LoadedProject, ProjectKind, ProjectLinkMeta, SourceOrigin};
pub use test_sources::is_test_source_file;
pub use workspace::{
    LoadedWorkspace, discover_run_project_in_workspace, discover_test_projects_in_workspace,
    discover_workspace_file, load_workspace, resolve_workspace_dependency_paths,
};
