use super::*;

#[test]
fn unreachable_unit_is_not_linked() {
    let cwd = create_temp_dir("run-unreachable-unit");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(&cwd.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    // This unit is valid but never imported — it should not affect the program
    write_text(
        &cwd.join("src/unused.fpas"),
        "unit App.Unused;\nfunction Unused(): integer;\nbegin\n  return 999\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
}

#[test]
fn unit_with_only_private_declarations_exports_nothing() {
    let cwd = create_temp_dir("run-only-private");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    // Import the unit but don't call anything — should succeed
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Internal;\nbegin\nend.\n",
    );
    write_text(
        &cwd.join("src/internal.fpas"),
        "\
unit App.Internal;

function Secret(): integer;
begin
  return 0
end;
",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
}

#[test]
fn calling_private_symbol_from_only_private_unit_fails() {
    let cwd = create_temp_dir("run-call-only-private");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Internal, Std.Console;\nbegin\n  WriteLn(Secret())\nend.\n",
    );
    write_text(
        &cwd.join("src/internal.fpas"),
        "\
unit App.Internal;

function Secret(): integer;
begin
  return 42
end;
",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Secret"),
        "error should mention the private symbol, got: {stderr_output}"
    );
}

#[test]
fn unused_import_does_not_cause_error() {
    let cwd = create_temp_dir("run-unused-import");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    // Import the unit but never call any of its functions
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Lib;\nbegin\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\nfunction Foo(): integer;\nbegin\n  return 1\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
}
