//! Project loading and user-unit linking for Functional Pascal.
//!
//! Documentation:
//! - `docs/pascal/program-structure/units.md`
//! - `docs/pascal/program-structure/projects.md`

mod common;
mod dependencies;
mod link;
mod loading;
mod model;
mod paths;
mod standard_library;
mod test_manifest;
mod test_sources;
mod workspace;

pub use link::{
    LinkedProgram, LinkedTestBundle, build_library_check_with_source_map,
    build_library_check_with_standard_library, build_program, build_program_with_source_map,
    build_program_with_standard_library, build_test_bundle_from_paths,
    build_test_bundle_from_paths_with_standard_library,
};
pub use loading::load_project;
pub use model::{LibraryExportPolicy, LoadedProject, ProjectKind, ProjectLinkMeta, SourceOrigin};
pub use standard_library::{StandardLibrary, load_standard_library};
pub use test_manifest::{TestFileOverride, TestManifest};
pub use test_sources::is_test_source_file;
pub use workspace::{
    LoadedWorkspace, discover_run_project_in_workspace, discover_test_projects_in_workspace,
    discover_workspace_file, load_workspace, resolve_workspace_dependency_paths,
};
