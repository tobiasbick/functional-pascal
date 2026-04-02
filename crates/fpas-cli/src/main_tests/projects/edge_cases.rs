use super::*;

#[test]
fn diamond_dependency_graph() {
    let cwd = create_temp_dir("run-diamond-deps");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.A, App.B, Std.Console;\nbegin\n  WriteLn(FromA() + FromB())\nend.\n",
    );
    write_text(
        &cwd.join("src/a.fpas"),
        "unit App.A;\nuses App.Shared;\nfunction FromA(): integer;\nbegin\n  return Base() + 1\nend;\n",
    );
    write_text(
        &cwd.join("src/b.fpas"),
        "unit App.B;\nuses App.Shared;\nfunction FromB(): integer;\nbegin\n  return Base() + 10\nend;\n",
    );
    write_text(
        &cwd.join("src/shared.fpas"),
        "unit App.Shared;\nfunction Base(): integer;\nbegin\n  return 100\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "211\n");
}

#[test]
fn three_unit_cyclic_dependency() {
    let cwd = create_temp_dir("run-three-cycle");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.A;\nbegin\nend.\n",
    );
    write_text(&cwd.join("src/a.fpas"), "unit App.A;\nuses App.B;\n");
    write_text(&cwd.join("src/b.fpas"), "unit App.B;\nuses App.C;\n");
    write_text(&cwd.join("src/c.fpas"), "unit App.C;\nuses App.A;\n");

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Cyclic unit dependency detected"),
        "expected cycle error, got: {stderr_output}"
    );
}

#[test]
fn duplicate_unit_names_in_different_files_rejected() {
    let cwd = create_temp_dir("run-duplicate-unit-name");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Lib;\nbegin\nend.\n",
    );
    write_text(
        &cwd.join("src/lib1.fpas"),
        "unit App.Lib;\nfunction Foo(): integer;\nbegin\n  return 1\nend;\n",
    );
    write_text(
        &cwd.join("src/lib2.fpas"),
        "unit App.Lib;\nfunction Bar(): integer;\nbegin\n  return 2\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Duplicate unit name"),
        "expected duplicate unit name error, got: {stderr_output}"
    );
}

#[test]
fn duplicate_uses_entries_are_harmless() {
    let cwd = create_temp_dir("run-dup-uses");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Lib, App.Lib, Std.Console;\nbegin\n  WriteLn(GetVal())\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\nfunction GetVal(): integer;\nbegin\n  return 7\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "7\n");
}

#[test]
fn single_segment_unit_name_compiles() {
    let cwd = create_temp_dir("run-single-seg-unit");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses Utils, Std.Console;\nbegin\n  WriteLn(GetNum())\nend.\n",
    );
    write_text(
        &cwd.join("src/utils.fpas"),
        "unit Utils;\nfunction GetNum(): integer;\nbegin\n  return 42\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "42\n");
}

#[test]
fn empty_unit_compiles_successfully() {
    let cwd = create_temp_dir("run-empty-unit");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Empty;\nbegin\nend.\n",
    );
    write_text(&cwd.join("src/empty.fpas"), "unit App.Empty;\n");

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
}

#[test]
fn self_import_reports_cycle() {
    let cwd = create_temp_dir("run-self-import");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.A;\nbegin\nend.\n",
    );
    write_text(&cwd.join("src/a.fpas"), "unit App.A;\nuses App.A;\n");

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Cyclic unit dependency detected"),
        "expected cycle error, got: {stderr_output}"
    );
}

#[test]
fn unit_name_resolved_case_insensitively() {
    let cwd = create_temp_dir("run-case-insensitive-unit");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    // uses clause has different casing than unit declaration
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses app.lib, Std.Console;\nbegin\n  WriteLn(GetValue())\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\nfunction GetValue(): integer;\nbegin\n  return 33\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "33\n");
}

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

private function Secret(): integer;
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

private function Secret(): integer;
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

#[test]
fn unit_name_is_resolved_from_declaration_not_file_path() {
    let cwd = create_temp_dir("run-unit-name-from-decl");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Tools, Std.Console;\nbegin\n  WriteLn(GetValue())\nend.\n",
    );
    write_text(
        &cwd.join("src/nested/mismatched_name.fpas"),
        "unit App.Tools;\nfunction GetValue(): integer;\nbegin\n  return 17\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "17\n");
}
