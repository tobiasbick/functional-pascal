use super::*;

/// Converts a path into TOML-friendly forward-slash form.
pub(super) fn toml_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Loads a project file and returns the parsed project.
///
/// Panics when the test fixture is expected to load successfully but does not.
pub(super) fn load_project_ok(project_file: &std::path::Path) -> fpas_project::LoadedProject {
    load_project(project_file).expect("project should load")
}

/// Loads a project file and returns the expected error string.
///
/// Panics when the test fixture is expected to fail loading but succeeds.
pub(super) fn load_project_error(project_file: &std::path::Path, context: &str) -> String {
    load_project(project_file).expect_err(context)
}
