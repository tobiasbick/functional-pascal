use super::*;

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
        "unit App.Lib;\npublic function Foo(): integer;\nbegin\n  return 1\nend;\n",
    );
    write_text(
        &cwd.join("src/lib2.fpas"),
        "unit App.Lib;\npublic function Bar(): integer;\nbegin\n  return 2\nend;\n",
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
        "unit App.Lib;\npublic function GetVal(): integer;\nbegin\n  return 7\nend;\n",
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
        "unit Utils;\npublic function GetNum(): integer;\nbegin\n  return 42\nend;\n",
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
        "unit App.Lib;\npublic function GetValue(): integer;\nbegin\n  return 33\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "33\n");
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
        "unit App.Tools;\npublic function GetValue(): integer;\nbegin\n  return 17\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "17\n");
}
