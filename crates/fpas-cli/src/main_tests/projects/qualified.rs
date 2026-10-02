use super::*;

#[test]
fn qualified_const_access_from_user_unit() {
    let cwd = create_temp_dir("run-qual-const");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Config as Config; uses Std.Console as Console;
begin
  Console.WriteLn(Config.MaxVal);
end program;
"#,
    );
    write_text(
        &cwd.join("src/config.fpas"),
        r#"unit App.Config;


  public const MaxVal: integer := 256;
end unit;

"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "256\n");
}

#[test]
fn case_insensitive_qualified_call() {
    let cwd = create_temp_dir("run-case-qual-call");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Lib as Lib; uses Std.Console as Console;
begin
  Console.WriteLn(Lib.GetValue());
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;
public function GetValue(): integer;
begin
  return 44;
end function;
end unit;

"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "44\n");
}

#[test]
fn qualified_name_call_to_user_unit_function() {
    let cwd = create_temp_dir("run-qualified-call");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Lib as Lib; uses Std.Console as Console;
begin
  Console.WriteLn(Lib.GetValue());
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;
public function GetValue(): integer;
begin
  return 77;
end function;
end unit;

"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "77\n");
}

#[test]
fn mixed_short_and_qualified_calls_in_same_program() {
    let cwd = create_temp_dir("run-mixed-calls");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Lib as Lib; uses Std.Console as Console;
begin
  Console.WriteLn(Lib.GetValue());
  Console.WriteLn(Lib.GetValue());
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;
public function GetValue(): integer;
begin
  return 55;
end function;
end unit;

"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "55\n55\n");
}

#[test]
fn deep_transitive_chain_four_levels() {
    let cwd = create_temp_dir("run-deep-transitive");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.A as A; uses Std.Console as Console;
begin
  Console.WriteLn(A.CallA());
end program;
"#,
    );
    write_text(
        &cwd.join("src/a.fpas"),
        r#"unit App.A;
uses App.B as B;
public function CallA(): integer;
begin
  return B.CallB() + 1;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/b.fpas"),
        r#"unit App.B;
uses App.C as C;
public function CallB(): integer;
begin
  return C.CallC() + 10;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/c.fpas"),
        r#"unit App.C;
public function CallC(): integer;
begin
  return 100;
end function;
end unit;

"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "111\n");
}
