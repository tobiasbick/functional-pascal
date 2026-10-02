use super::*;

#[test]
fn unreachable_unit_is_not_linked() {
    let cwd = create_temp_dir("run-unreachable-unit");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
begin null;
end program;
"#,
    );
    // This unit is valid but never imported — it should not affect the program
    write_text(
        &cwd.join("src/unused.fpas"),
        r#"unit App.Unused;
function Unused(): integer;
begin
  return 999;
end function;
end unit;

"#,
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
        r#"program Main;
uses App.Internal as Internal;
begin null;
end program;
"#,
    );
    write_text(
        &cwd.join("src/internal.fpas"),
        r#"unit App.Internal;

function Secret(): integer;
begin
  return 0;
end function;
end unit;

"#,
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
        r#"program Main;
uses App.Internal as Internal; uses Std.Console as Console;
begin
  Console.WriteLn(Secret());
end program;
"#,
    );
    write_text(
        &cwd.join("src/internal.fpas"),
        r#"unit App.Internal;

function Secret(): integer;
begin
  return 42;
end function;
end unit;

"#,
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
        r#"program Main;
uses App.Lib as Lib;
begin null;
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;
function Foo(): integer;
begin
  return 1;
end function;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
}
