use super::*;

#[test]
fn unknown_std_unit_rejected_with_available_units_hint() {
    let (exit_code, _, stderr_output) = support::run_source_and_capture_output(
        "unknown_std.fpas",
        "program Test;\nuses Std.Nonexistent;\nbegin\nend.\n",
    );

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Unknown standard library unit"),
        "expected unknown std unit error, got: {stderr_output}"
    );
    // The error message should hint at valid units
    assert!(
        stderr_output.contains("Std.Console") || stderr_output.contains("Available"),
        "expected available units hint, got: {stderr_output}"
    );
}

#[test]
fn bare_std_uses_rejected() {
    let (exit_code, _, stderr_output) = support::run_source_and_capture_output(
        "bare_std.fpas",
        "program Test;\nuses Std;\nbegin\nend.\n",
    );

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Std"),
        "expected Std namespace error, got: {stderr_output}"
    );
}

#[test]
fn std_uses_case_insensitive() {
    let (exit_code, stdout_output, stderr_output) = support::run_source_and_capture_output(
        "std_case.fpas",
        "program Test;\nuses std.console;\nbegin\n  WriteLn(42)\nend.\n",
    );

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "42\n");
}

#[test]
fn std_unit_with_extra_segments_rejected() {
    let (exit_code, _, stderr_output) = support::run_source_and_capture_output(
        "std_extra_seg.fpas",
        "program Test;\nuses Std.Console.Extra;\nbegin\nend.\n",
    );

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Std"),
        "expected Std segment error, got: {stderr_output}"
    );
}

#[test]
fn user_unit_cannot_use_std_namespace() {
    let cwd = create_temp_dir("run-user-std-ns");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses Std.MyLib;\nbegin\nend.\n",
    );
    write_text(
        &cwd.join("src/mylib.fpas"),
        "unit Std.MyLib;\nfunction Foo(): integer;\nbegin\n  return 1\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    // Std.MyLib is treated as a standard lib unit and rejected as unknown
    assert_eq!(exit_code, 1, "stderr: {stderr_output}");
    assert!(
        stderr_output.contains("Unknown standard library unit") || stderr_output.contains("Std"),
        "expected Std namespace error, got: {stderr_output}"
    );
}
