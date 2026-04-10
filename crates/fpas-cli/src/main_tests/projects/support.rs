use crate::test_support::{write_library_fpasprj, write_program_fpasprj};

pub(super) use crate::main_tests::support::run_cli_and_capture_output;

/// Writes a program project manifest for CLI integration tests.
pub(super) fn write_program_project_file(
    project_file: &std::path::Path,
    main: &str,
    include: &[&str],
) {
    write_program_fpasprj(project_file, main, include);
}

/// Writes a library project manifest for CLI integration tests.
pub(super) fn write_library_project_file(project_file: &std::path::Path, include: &[&str]) {
    write_library_fpasprj(project_file, include);
}
