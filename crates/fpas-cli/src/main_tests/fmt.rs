use super::support::run_cli_args_and_capture_output;
use super::{create_temp_dir, write_text};
use crate::cli_fmt::EXIT_WOULD_CHANGE;
use std::fs;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

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
fn fmt_cli_formats_two_explicit_source_files() {
    let cwd = create_temp_dir("fmt-two-files");
    let first = cwd.join("a.fpas");
    let second = cwd.join("b.fpas");
    let messy = "program Hello; uses Std.Console; begin WriteLn('hi') end.";
    write_text(&first, messy);
    write_text(&second, messy);

    let (exit_code, _, stderr_output) = run_cli_args_and_capture_output(
        &[
            String::from("fmt"),
            first.to_string_lossy().to_string(),
            second.to_string_lossy().to_string(),
        ],
        &cwd,
    );

    let formatted_first = fs::read_to_string(&first).expect("first file must exist");
    let formatted_second = fs::read_to_string(&second).expect("second file must exist");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert!(formatted_first.contains("program Hello;\n\nuses Std.Console;\n\nbegin\n"));
    assert!(formatted_second.contains("program Hello;\n\nuses Std.Console;\n\nbegin\n"));
}

#[test]
fn fmt_cli_project_includes_its_program_main() {
    let cwd = create_temp_dir("fmt-program-project");
    let project = cwd.join("app.fpasprj");
    let main = cwd.join("src/main.fpas");
    write_text(
        &project,
        r#"[project]
name = "fmt-app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&main, "program Main; begin var Value:integer:=1 end.");

    let (exit_code, _, stderr_output) = run_cli_args_and_capture_output(
        &[String::from("fmt"), project.to_string_lossy().to_string()],
        &cwd,
    );
    let formatted = fs::read_to_string(&main).expect("main source must exist");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(
        formatted,
        "program Main;\n\nbegin\n  var Value: integer := 1\nend.\n"
    );
}

#[test]
fn fmt_cli_stdout_does_not_modify_file_on_disk() {
    let cwd = create_temp_dir("fmt-stdout");
    let source_path = cwd.join("hello.fpas");
    let original = "program Hello; uses Std.Console; begin WriteLn('hi') end.";
    write_text(&source_path, original);

    let (exit_code, stdout_output, stderr_output) = run_cli_args_and_capture_output(
        &[
            String::from("fmt"),
            String::from("--stdout"),
            source_path.to_string_lossy().to_string(),
        ],
        &cwd,
    );

    let unchanged = fs::read_to_string(&source_path).expect("source file must exist");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(unchanged, original);
    assert!(stdout_output.contains("program Hello;\n\nuses Std.Console;\n\nbegin\n"));
}

#[test]
fn fmt_cli_check_list_prints_dirty_paths_only() {
    let cwd = create_temp_dir("fmt-check-list");
    let dirty = cwd.join("dirty.fpas");
    let clean = cwd.join("clean.fpas");
    write_text(
        &dirty,
        "program Dirty; uses Std.Console; begin WriteLn('dirty') end.",
    );
    write_text(
        &clean,
        "program Clean;\n\nuses Std.Console;\n\nbegin\n  WriteLn('clean')\nend.\n",
    );

    let (exit_code, stdout_output, stderr_output) = run_cli_args_and_capture_output(
        &[
            String::from("fmt"),
            String::from("--check"),
            String::from("--list"),
            dirty.to_string_lossy().to_string(),
            clean.to_string_lossy().to_string(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, EXIT_WOULD_CHANGE, "stderr: {stderr_output}");
    assert_eq!(stdout_output.trim(), dirty.display().to_string());
}

#[test]
fn fmt_cli_expands_glob_pattern() {
    let cwd = create_temp_dir("fmt-glob");
    let src_dir = cwd.join("src");
    fs::create_dir_all(&src_dir).expect("src directory must exist");
    let nested = src_dir.join("nested.fpas");
    write_text(
        &nested,
        "program Nested; uses Std.Console; begin WriteLn('nested') end.",
    );

    let (exit_code, _, stderr_output) = run_cli_args_and_capture_output(
        &[String::from("fmt"), String::from("src/**/*.fpas")],
        &cwd,
    );

    let formatted = fs::read_to_string(&nested).expect("nested file must exist");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert!(formatted.contains("program Nested;\n\nuses Std.Console;\n\nbegin\n"));
}

#[cfg(unix)]
#[test]
fn fmt_cli_expands_glob_below_non_utf8_working_directory() {
    let mut directory_name = format!("fpas-fmt-non-utf8-{}-", std::process::id()).into_bytes();
    directory_name.push(0xff);
    let cwd = std::env::temp_dir().join(std::ffi::OsString::from_vec(directory_name));
    let source = cwd.join("src/nested.fpas");
    fs::create_dir_all(source.parent().expect("source must have a parent"))
        .expect("source directory must be created");
    write_text(
        &source,
        "program Nested; uses Std.Console; begin WriteLn('nested') end.",
    );

    let (exit_code, _, stderr_output) = run_cli_args_and_capture_output(
        &[String::from("fmt"), String::from("src/**/*.fpas")],
        &cwd,
    );
    let formatted = fs::read_to_string(&source).expect("source must remain readable");
    fs::remove_dir_all(&cwd).expect("fixture must be removed");

    assert_eq!(
        (
            exit_code,
            formatted.contains("program Nested;\n\nuses Std.Console;\n\nbegin\n")
        ),
        (0, true),
        "stderr: {stderr_output}"
    );
}

#[test]
fn fmt_cli_does_not_follow_file_symlinks() {
    let cwd = create_temp_dir("fmt-symlink");
    let outside = create_temp_dir("fmt-symlink-target");
    let target = outside.join("outside.fpas");
    let link = cwd.join("linked.fpas");
    let original = "program Outside; begin end.";
    write_text(&target, original);

    #[cfg(unix)]
    let link_result = std::os::unix::fs::symlink(&target, &link);
    #[cfg(windows)]
    let link_result = std::os::windows::fs::symlink_file(&target, &link);
    if let Err(error) = link_result {
        #[cfg(windows)]
        if error.kind() == std::io::ErrorKind::PermissionDenied
            || error.raw_os_error() == Some(1314)
        {
            fs::remove_dir_all(&cwd).expect("temp directory must be removed");
            fs::remove_dir_all(&outside).expect("target directory must be removed");
            return;
        }
        panic!("file symlink fixture failed: {error}");
    }

    let (glob_exit, _, glob_stderr) =
        run_cli_args_and_capture_output(&[String::from("fmt"), String::from("*.fpas")], &cwd);
    let (direct_exit, _, direct_stderr) = run_cli_args_and_capture_output(
        &[String::from("fmt"), link.to_string_lossy().into_owned()],
        &cwd,
    );
    let unchanged = fs::read_to_string(&target).expect("target must remain readable");
    fs::remove_file(&link).expect("symlink must be removed");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
    fs::remove_dir_all(&outside).expect("target directory must be removed");

    assert_eq!(glob_exit, 1, "{glob_stderr}");
    assert!(glob_stderr.contains("no regular `.fpas` files"));
    assert_eq!(direct_exit, 1, "{direct_stderr}");
    assert!(direct_stderr.contains("symbolic link"));
    assert_eq!(unchanged, original);
}

#[test]
fn fmt_cli_rejects_stdout_with_check() {
    let cwd = create_temp_dir("fmt-stdout-check");
    let source_path = cwd.join("hello.fpas");
    write_text(
        &source_path,
        "program Hello; uses Std.Console; begin WriteLn('hi') end.",
    );

    let (exit_code, _, stderr_output) = run_cli_args_and_capture_output(
        &[
            String::from("fmt"),
            String::from("--stdout"),
            String::from("--check"),
            source_path.to_string_lossy().to_string(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("cannot be combined"));
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
