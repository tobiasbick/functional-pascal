//! Project loading and user-unit linking for Functional Pascal.
//!
//! Documentation:
//! - `docs/pascal/program-structure/units.md`
//! - `docs/pascal/program-structure/projects.md`

mod common;
mod dependencies;
mod loading;
mod model;
mod paths;
mod standard_library;
mod test_manifest;
mod test_sources;
mod unit_graph;
mod workspace;

pub use loading::load_project;
pub use model::{LibraryExportPolicy, LoadedProject, ProjectKind, ProjectLinkMeta, SourceOrigin};
pub use standard_library::{StandardLibrary, load_standard_library};
pub use test_manifest::{TestFileOverride, TestManifest};
pub use test_sources::is_test_source_file;
pub use unit_graph::{
    ResolvedUnitGraph, UnitGraph, UnitNode, build_unit_graph, build_unit_graph_for_program,
    build_unit_graph_for_program_with_standard_library, build_unit_graph_with_standard_library,
    resolve_library_units, resolve_program_units,
};
pub use workspace::{
    LoadedWorkspace, discover_run_project_in_workspace, discover_test_projects_in_workspace,
    discover_workspace_file, load_workspace, resolve_workspace_dependency_paths,
};
