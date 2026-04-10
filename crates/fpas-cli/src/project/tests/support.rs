use super::*;
use fpas_parser::Program;

/// Converts a path into TOML-friendly forward-slash form.
pub(super) fn toml_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Writes a standard program project manifest with the provided source includes.
pub(super) fn write_program_project_file(project_file: &std::path::Path, include: &[&str]) {
    crate::test_support::write_program_fpasprj(project_file, "src/main.fpas", include);
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

/// Loads a project and builds its linked program.
///
/// The project fixture must define a main file and link successfully.
pub(super) fn load_and_build_program(project_file: &std::path::Path) -> Program {
    let loaded = load_project_ok(project_file);
    build_program(
        loaded.main.as_deref().expect("main path must exist"),
        &loaded.source_files,
    )
    .expect("project should link")
}

/// Asserts that a qualified designator contains exactly the expected identifiers.
pub(super) fn assert_qualified_designator(parts: &[DesignatorPart], expected: &[&str]) {
    assert_eq!(parts.len(), expected.len());
    for (part, expected_name) in parts.iter().zip(expected.iter()) {
        match part {
            DesignatorPart::Ident(actual, _) => assert_eq!(actual, expected_name),
            other => panic!("expected identifier part, got {other:?}"),
        }
    }
}

/// Asserts that a designator contains a single identifier part.
pub(super) fn assert_single_ident(parts: &[DesignatorPart], expected: &str) {
    assert_qualified_designator(parts, &[expected]);
}
