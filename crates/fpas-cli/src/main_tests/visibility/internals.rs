use super::*;

#[test]
fn private_function_callable_within_same_unit() {
    let cwd = create_temp_dir("vis-private-internal");
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(GetValue())\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "\
unit App.Lib;

private function Secret(): integer;
begin
  return 42
end;

function GetValue(): integer;
begin
  return Secret()
end;
",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0);
    assert!(stderr_output.is_empty());
    assert_eq!(stdout_output, "42\n");
}

#[test]
fn mixed_public_private_only_public_exported() {
    let cwd = create_temp_dir("vis-mixed");
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(PublicFn())\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "\
unit App.Lib;

private function Helper(): integer;
begin
  return 10
end;

function PublicFn(): integer;
begin
  return Helper() + 5
end;
",
    );

    let (exit_code, stdout_output, _) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "15\n");
}

#[test]
fn private_visibility_in_program_is_rejected() {
    let (exit_code, _, stderr_output) = support::run_source_and_capture_output(
        "private_program_visibility.fpas",
        "\
program Main;

private var
  X: integer := 1;

begin
  WriteLn(X)
end.
",
    );

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("`private` is not valid in a `program` file"),
        "expected program visibility error, got: {stderr_output}"
    );
}

#[test]
fn public_visibility_in_program_is_rejected() {
    let (exit_code, _, stderr_output) = support::run_source_and_capture_output(
        "public_program_visibility.fpas",
        "\
program Main;

public var
  X: integer := 1;

begin
  WriteLn(X)
end.
",
    );

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("`public` is not valid in a `program` file"),
        "expected program visibility error, got: {stderr_output}"
    );
}
