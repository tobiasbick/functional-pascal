use super::*;

pub(super) use crate::main_tests::support::run_cli_and_capture_output;

/// Writes a program project manifest for CLI integration tests.
pub(super) fn write_program_project_file(
    project_file: &std::path::Path,
    main: &str,
    include: &[&str],
) {
    write_project_file(project_file, "app", "program", Some(main), include);
}

/// Writes a library project manifest for CLI integration tests.
pub(super) fn write_library_project_file(project_file: &std::path::Path, include: &[&str]) {
    write_project_file(project_file, "lib", "library", None, include);
}

fn write_project_file(
    project_file: &std::path::Path,
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
