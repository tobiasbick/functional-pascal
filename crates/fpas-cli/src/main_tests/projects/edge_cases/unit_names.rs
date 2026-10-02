use super::*;

#[test]
fn duplicate_unit_names_in_different_files_rejected() {
    let cwd = create_temp_dir("run-duplicate-unit-name");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Lib as Lib;
begin null;
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib1.fpas"),
        r#"unit App.Lib;
public function Foo(): integer;
begin
  return 1;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/lib2.fpas"),
        r#"unit App.Lib;
public function Bar(): integer;
begin
  return 2;
end function;
end unit;

"#,
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
fn duplicate_uses_entries_are_rejected() {
    let cwd = create_temp_dir("run-dup-uses");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Lib as Lib; uses App.Lib as Lib2; uses Std.Console as Console;
begin
  Console.WriteLn(Lib.GetVal());
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;
public function GetVal(): integer;
begin
  return 7;
end function;
end unit;

"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1, "stderr: {stderr_output}");
    assert!(stdout_output.is_empty());
    assert!(stderr_output.contains("Unit `App.Lib` is imported more than once"));
}

#[test]
fn single_segment_unit_name_compiles() {
    let cwd = create_temp_dir("run-single-seg-unit");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses Utils as Utils; uses Std.Console as Console;
begin
  Console.WriteLn(Utils.GetNum());
end program;
"#,
    );
    write_text(
        &cwd.join("src/utils.fpas"),
        r#"unit Utils;
public function GetNum(): integer;
begin
  return 42;
end function;
end unit;

"#,
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
        r#"program Main;
uses App.Empty as Empty;
begin null;
end program;
"#,
    );
    write_text(
        &cwd.join("src/empty.fpas"),
        r#"unit App.Empty;
end unit;

"#,
    );

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
        r#"program Main;
uses app.lib as lib; uses Std.Console as Console;
begin
  Console.WriteLn(lib.GetValue());
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;
public function GetValue(): integer;
begin
  return 33;
end function;
end unit;

"#,
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
        r#"program Main;
uses App.Tools as Tools; uses Std.Console as Console;
begin
  Console.WriteLn(Tools.GetValue());
end program;
"#,
    );
    write_text(
        &cwd.join("src/nested/mismatched_name.fpas"),
        r#"unit App.Tools;
public function GetValue(): integer;
begin
  return 17;
end function;
end unit;

"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "17\n");
}
