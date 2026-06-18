use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) fn create_temp_dir(prefix: &str) -> PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let suffix = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "fpas-tests-{prefix}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("temp directory must be created");
    dir
}

pub(crate) fn write_file(path: &Path) {
    fs::write(path, "").expect("test file must be created");
}

pub(crate) fn write_text(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directories must be created");
    }
    fs::write(path, text).expect("test file must be created");
}

/// Writes a `.fpasprj` manifest for tests (`kind = "program"`).
///
/// Spec: [Projects & CLI](../../../docs/pascal/program-structure/cli.md).
pub(crate) fn write_program_fpasprj(project_file: &Path, main: &str, include: &[&str]) {
    write_fpasprj(
        project_file,
        "app",
        "program",
        Some(main),
        include,
        &[],
        &[],
        None,
    );
}

/// Writes a `.fpasprj` manifest for tests (`kind = "library"`).
///
/// Spec: [Projects & CLI](../../../docs/pascal/program-structure/cli.md).
pub(crate) fn write_library_fpasprj(project_file: &Path, include: &[&str]) {
    write_fpasprj(
        project_file,
        "lib",
        "library",
        None,
        include,
        &[],
        &[],
        None,
    );
}

/// Writes a library manifest with an `[exports].units` list.
///
/// Documentation: `docs/pascal/program-structure/projects.md`
pub(crate) fn write_library_fpasprj_with_exports(
    project_file: &Path,
    include: &[&str],
    export_units: &[&str],
) {
    write_fpasprj(
        project_file,
        "lib",
        "library",
        None,
        include,
        &[],
        &[],
        Some(export_units),
    );
}

pub(crate) fn write_program_fpasprj_with_deps(
    project_file: &Path,
    main: &str,
    include: &[&str],
    dependencies: &[&str],
) {
    write_fpasprj(
        project_file,
        "app",
        "program",
        Some(main),
        include,
        dependencies,
        &[],
        None,
    );
}

pub(crate) fn write_program_fpasprj_with_workspace_deps(
    project_file: &Path,
    main: &str,
    include: &[&str],
    workspace_deps: &[&str],
) {
    write_fpasprj(
        project_file,
        "app",
        "program",
        Some(main),
        include,
        &[],
        workspace_deps,
        None,
    );
}

pub(crate) fn write_library_fpasprj_with_deps(
    project_file: &Path,
    include: &[&str],
    dependencies: &[&str],
) {
    write_fpasprj(
        project_file,
        "lib",
        "library",
        None,
        include,
        dependencies,
        &[],
        None,
    );
}

#[allow(clippy::too_many_arguments)]
fn write_fpasprj(
    project_file: &Path,
    name: &str,
    kind: &str,
    main: Option<&str>,
    include: &[&str],
    dependencies: &[&str],
    workspace_dependencies: &[&str],
    export_units: Option<&[&str]>,
) {
    let include_entries = include
        .iter()
        .map(|entry| format!("\"{entry}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let main_entry = match main {
        Some(main) => format!("main = \"{main}\"\n\n"),
        None => String::new(),
    };
    let dependencies_section = format_dependency_section(dependencies, workspace_dependencies);
    let exports_section = format_exports_section(export_units);

    write_text(
        project_file,
        &format!(
            r#"[project]
name = "{name}"
kind = "{kind}"
{main_entry}[sources]
include = [{include_entries}]{dependencies_section}{exports_section}"#
        ),
    );
}

fn format_exports_section(export_units: Option<&[&str]>) -> String {
    let Some(units) = export_units else {
        return String::new();
    };
    let entries = units
        .iter()
        .map(|unit| format!("\"{unit}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("\n\n[exports]\nunits = [{entries}]\n")
}

fn format_dependency_section(dependencies: &[&str], workspace_dependencies: &[&str]) -> String {
    if dependencies.is_empty() && workspace_dependencies.is_empty() {
        return String::new();
    }

    let mut section = String::from("\n\n[dependencies]\n");
    if !dependencies.is_empty() {
        let dependency_entries = dependencies
            .iter()
            .map(|entry| format!("\"{entry}\""))
            .collect::<Vec<_>>()
            .join(", ");
        section.push_str(&format!("projects = [{dependency_entries}]\n"));
    }
    if !workspace_dependencies.is_empty() {
        let workspace_entries = workspace_dependencies
            .iter()
            .map(|entry| format!("\"{entry}\""))
            .collect::<Vec<_>>()
            .join(", ");
        section.push_str(&format!("workspace = [{workspace_entries}]\n"));
    }

    section
}
