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
/// Spec: [Projects & CLI](../../../docs/pascal/10-projects.md).
pub(crate) fn write_program_fpasprj(project_file: &Path, main: &str, include: &[&str]) {
    write_fpasprj(project_file, "app", "program", Some(main), include);
}

/// Writes a `.fpasprj` manifest for tests (`kind = "library"`).
///
/// Spec: [Projects & CLI](../../../docs/pascal/10-projects.md).
pub(crate) fn write_library_fpasprj(project_file: &Path, include: &[&str]) {
    write_fpasprj(project_file, "lib", "library", None, include);
}

fn write_fpasprj(
    project_file: &Path,
    name: &str,
    kind: &str,
    main: Option<&str>,
    include: &[&str],
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

    write_text(
        project_file,
        &format!(
            r#"[project]
name = "{name}"
kind = "{kind}"
{main_entry}[sources]
include = [{include_entries}]
"#
        ),
    );
}
