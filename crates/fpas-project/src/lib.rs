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
mod workspace;

pub use link::{
    LinkedProgram, build_library_check_with_source_map, build_program,
    build_program_with_source_map,
};
pub use loading::load_project;
pub use model::{LoadedProject, ProjectKind};
pub use workspace::{
    LoadedWorkspace, discover_run_project_in_workspace, discover_workspace_file, load_workspace,
};
