use super::support::run_cli_args_and_capture_output;
use super::{create_temp_dir, write_text};
use crate::cli_fmt::EXIT_WOULD_CHANGE;
use std::fs;

#[test]
fn fmt_cli_formats_source_file_in_place() {
    let cwd = create_temp_dir("fmt-source");
    let source_path = cwd.join("hello.fpas");
    write_text(
        &source_path,
        "program Hello; uses Std.Console; begin WriteLn('hi') end.",
    );

    let (exit_code, _, stderr_output) = run_cli_args_and_capture_output(
        &[
            String::from("fmt"),
            source_path.to_string_lossy().to_string(),
        ],
        &cwd,
    );

    let formatted = fs::read_to_string(&source_path).expect("formatted file must exist");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert!(stderr_output.is_empty());
    assert!(formatted.contains("program Hello;\n\nuses Std.Console;\n\nbegin\n"));
}

#[test]
fn fmt_cli_check_reports_unformatted_file() {
    let cwd = create_temp_dir("fmt-check-dirty");
    let source_path = cwd.join("hello.fpas");
    write_text(
        &source_path,
        "program Hello; uses Std.Console; begin WriteLn('hi') end.",
    );

    let (exit_code, _, stderr_output) = run_cli_args_and_capture_output(
        &[
            String::from("fmt"),
            String::from("--check"),
            source_path.to_string_lossy().to_string(),
        ],
        &cwd,
    );

    let unchanged = fs::read_to_string(&source_path).expect("source file must exist");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, EXIT_WOULD_CHANGE, "stderr: {stderr_output}");
    assert!(unchanged.contains("program Hello; uses Std.Console;"));
}

#[test]
fn fmt_cli_check_passes_on_canonical_file() {
    let cwd = create_temp_dir("fmt-check-clean");
    let source_path = cwd.join("hello.fpas");
    write_text(
        &source_path,
        "program Hello;\n\nuses Std.Console;\n\nbegin\n  WriteLn('hi')\nend.\n",
    );

    let (exit_code, _, stderr_output) = run_cli_args_and_capture_output(
        &[
            String::from("fmt"),
            String::from("--check"),
            source_path.to_string_lossy().to_string(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
}

#[test]
fn fmt_cli_rejects_program_args_after_separator() {
    let cwd = create_temp_dir("fmt-program-args");
    let (exit_code, _, stderr_output) = run_cli_args_and_capture_output(
        &[String::from("fmt"), String::from("--"), String::from("arg")],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("fpas fmt"));
    assert!(stderr_output.contains("does not accept program arguments"));
}
