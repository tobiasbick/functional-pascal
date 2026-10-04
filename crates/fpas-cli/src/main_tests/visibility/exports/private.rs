use super::*;

#[test]
fn private_function_not_exported() {
    let cwd = create_temp_dir("vis-private-fn");
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
        r#"program Main;
uses App.Lib as Lib; uses Std.Console as Console;
begin
  Console.WriteLn(Secret());
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;

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
        "error should mention the private symbol name"
    );
}

#[test]
fn private_function_not_exported_by_qualified_name() {
    let cwd = create_temp_dir("vis-private-fn-qualified");
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
        r#"program Main;
uses App.Lib as Lib; uses Std.Console as Console;
begin
  Console.WriteLn(Lib.Secret());
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;

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
        stderr_output.contains("App.Lib.Secret"),
        "error should mention the qualified private symbol, got: {stderr_output}"
    );
}

#[test]
fn private_const_not_exported() {
    let cwd = create_temp_dir("vis-private-const");
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
        r#"program Main;
uses App.Lib as Lib; uses Std.Console as Console;
begin
  Console.WriteLn(Secret);
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;


  const Secret: integer := 42;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Secret"),
        "error should mention the const name"
    );
}

#[test]
fn private_procedure_not_exported() {
    let cwd = create_temp_dir("vis-private-proc");
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
        r#"program Main;
uses App.Lib as Lib;
begin
  DoSecret();
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;

procedure DoSecret();
begin null;
end procedure;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("DoSecret"),
        "error should mention the procedure name, got: {stderr_output}"
    );
}

#[test]
fn private_type_not_exported() {
    let cwd = create_temp_dir("vis-private-type");
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
        r#"program Main;
uses App.Lib as Lib;
begin
  var P: SecretPoint := SecretPoint(X := 1, Y := 2);
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;


  type SecretPoint = record
    X: integer;
    Y: integer;
  end record;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("SecretPoint"),
        "error should mention the type name, got: {stderr_output}"
    );
}

#[test]
fn private_type_not_exported_by_qualified_name() {
    let cwd = create_temp_dir("vis-private-type-qualified");
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
        r#"program Main;
uses App.Lib as Lib;
begin
  var P: Lib.SecretPoint := Lib.SecretPoint(X := 1, Y := 2);
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;


  type SecretPoint = record
    X: integer;
    Y: integer;
  end record;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("App.Lib.SecretPoint"),
        "error should mention the qualified type, got: {stderr_output}"
    );
}

#[test]
fn private_var_not_exported() {
    let cwd = create_temp_dir("vis-private-var");
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
        r#"program Main;
uses App.Lib as Lib; uses Std.Console as Console;
begin
  Console.WriteLn(Secret);
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;


  var Secret: integer := 42;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Secret"),
        "error should mention the var name, got: {stderr_output}"
    );
}

#[test]
fn private_mutable_var_not_exported() {
    let cwd = create_temp_dir("vis-private-mutvar");
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
        r#"program Main;
uses App.Lib as Lib; uses Std.Console as Console;
begin
  Console.WriteLn(Counter);
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;


  mutable var Counter: integer := 0;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Counter"),
        "error should mention the mutable var name, got: {stderr_output}"
    );
}

#[test]
fn private_var_not_exported_by_qualified_name() {
    let cwd = create_temp_dir("vis-private-var-qual");
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
        r#"program Main;
uses App.Lib as Lib; uses Std.Console as Console;
begin
  Console.WriteLn(Lib.Secret);
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;


  var Secret: integer := 42;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("App.Lib.Secret"),
        "error should mention the qualified var, got: {stderr_output}"
    );
}

#[test]
fn private_const_not_exported_by_qualified_name() {
    let cwd = create_temp_dir("vis-private-const-qual");
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
        r#"program Main;
uses App.Lib as Lib; uses Std.Console as Console;
begin
  Console.WriteLn(Lib.Secret);
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;


  const Secret: integer := 42;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("App.Lib.Secret"),
        "error should mention the qualified const, got: {stderr_output}"
    );
}

#[test]
fn private_procedure_not_exported_by_qualified_name() {
    let cwd = create_temp_dir("vis-private-proc-qual");
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
        r#"program Main;
uses App.Lib as Lib;
begin
  Lib.DoSecret();
end program;
"#,
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        r#"unit App.Lib;

procedure DoSecret();
begin null;
end procedure;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Private unit members are not visible outside their unit"),
        "error should hint at private visibility, got: {stderr_output}"
    );
}
