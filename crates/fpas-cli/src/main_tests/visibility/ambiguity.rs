use super::*;

#[test]
fn unqualified_call_with_three_imports_is_unknown() {
    let cwd = create_temp_dir("vis-ambig-three");
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
uses App.A as A; uses App.B as B; uses App.C as C;
begin
  Compute();
end program;
"#,
    );
    write_text(
        &cwd.join("src/a.fpas"),
        r#"unit App.A;
public function Compute(): integer;
begin
  return 1;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/b.fpas"),
        r#"unit App.B;
public function Compute(): integer;
begin
  return 2;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/c.fpas"),
        r#"unit App.C;
public function Compute(): integer;
begin
  return 3;
end function;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Unknown procedure `Compute`"),
        "expected unknown-name error, got: {stderr_output}"
    );
}

#[test]
fn unknown_name_error_mentions_alias_qualified_alternatives() {
    let cwd = create_temp_dir("vis-ambig-hint");
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
uses App.X as X; uses App.Y as Y;
begin
  Foo();
end program;
"#,
    );
    write_text(
        &cwd.join("src/x.fpas"),
        r#"unit App.X;
public function Foo(): integer;
begin
  return 1;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/y.fpas"),
        r#"unit App.Y;
public function Foo(): integer;
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
        stderr_output.contains("X.Foo") && stderr_output.contains("Y.Foo"),
        "error should mention qualified alternatives, got: {stderr_output}"
    );
}

#[test]
fn public_member_remains_accessible_through_its_alias() {
    let cwd = create_temp_dir("vis-no-ambiguity");
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
uses App.A as A; uses App.B as B; uses Std.Console as Console;
begin
  Console.WriteLn(B.Compute());
end program;
"#,
    );
    write_text(
        &cwd.join("src/a.fpas"),
        r#"unit App.A;

function Compute(): integer;
begin
  return 1;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/b.fpas"),
        r#"unit App.B;

public function Compute(): integer;
begin
  return 2;
end function;
end unit;

"#,
    );

    let (exit_code, stdout_output, _) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "2\n");
}

#[test]
fn ambiguity_resolved_by_qualified_name() {
    let cwd = create_temp_dir("vis-ambig-qualified");
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
    // Two units export the same short name — use qualified names to disambiguate
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Math as Math; uses App.Advanced as Advanced; uses Std.Console as Console;
begin
  Console.WriteLn(Math.Add(1, 2));
  Console.WriteLn(Advanced.Add(10, 20));
end program;
"#,
    );
    write_text(
        &cwd.join("src/math.fpas"),
        r#"unit App.Math;
public function Add(A: integer; B: integer): integer;
begin
  return A + B;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/advanced.fpas"),
        r#"unit App.Advanced;
public function Add(A: integer; B: integer): integer;
begin
  return A * B;
end function;
end unit;

"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "3\n200\n");
}

#[test]
fn no_error_at_uses_site_when_ambiguous_name_not_used() {
    let cwd = create_temp_dir("vis-unused-ambiguity");
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
    // Import two units with conflicting short names but never use the ambiguous name
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Math as Math; uses App.Advanced as Advanced;
begin null;
end program;
"#,
    );
    write_text(
        &cwd.join("src/math.fpas"),
        r#"unit App.Math;
public function Add(A: integer; B: integer): integer;
begin
  return A + B;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/advanced.fpas"),
        r#"unit App.Advanced;
public function Add(A: integer; B: integer): integer;
begin
  return A * B;
end function;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    // No error because the ambiguous short name `Add` is never referenced
    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
}
