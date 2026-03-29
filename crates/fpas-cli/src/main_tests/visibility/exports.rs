use super::*;

#[test]
fn public_function_is_callable_from_main() {
    let cwd = create_temp_dir("vis-public-fn");
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
        "unit App.Lib;\n\npublic function GetValue(): integer;\nbegin\n  return 42\nend;\n",
    );

    let (exit_code, stdout_output, _) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "42\n");
}

#[test]
fn default_visibility_function_is_callable() {
    let cwd = create_temp_dir("vis-default-fn");
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
        "unit App.Lib;\n\nfunction GetValue(): integer;\nbegin\n  return 99\nend;\n",
    );

    let (exit_code, stdout_output, _) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "99\n");
}

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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(Secret())\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nprivate function Secret(): integer;\nbegin\n  return 42\nend;\n",
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(App.Lib.Secret())\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nprivate function Secret(): integer;\nbegin\n  return 42\nend;\n",
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(Secret)\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nprivate const\n  Secret: integer := 42;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Secret"),
        "error should mention the private const name"
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
        "program Main;\nuses App.Lib;\nbegin\n  DoSecret()\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nprivate procedure DoSecret();\nbegin\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("DoSecret"),
        "error should mention the private procedure name, got: {stderr_output}"
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
        "\
program Main;
uses App.Lib;
begin
  var P: SecretPoint := SecretPoint { X: 1; Y: 2 };
end.
",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "\
unit App.Lib;

private type
  SecretPoint = record
    X: integer;
    Y: integer;
  end;
",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("SecretPoint"),
        "error should mention the private type name, got: {stderr_output}"
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
        "\
program Main;
uses App.Lib;
begin
  var P: App.Lib.SecretPoint := App.Lib.SecretPoint { X: 1; Y: 2 };
end.
",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "\
unit App.Lib;

private type
  SecretPoint = record
    X: integer;
    Y: integer;
  end;
",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("App.Lib.SecretPoint"),
        "error should mention the qualified private type, got: {stderr_output}"
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(Secret)\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nprivate var\n  Secret: integer := 42;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Secret"),
        "error should mention the private var name, got: {stderr_output}"
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(Counter)\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nprivate mutable var\n  Counter: integer := 0;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Counter"),
        "error should mention the private mutable var name, got: {stderr_output}"
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(App.Lib.Secret)\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nprivate var\n  Secret: integer := 42;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("App.Lib.Secret"),
        "error should mention the qualified private var, got: {stderr_output}"
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(App.Lib.Secret)\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nprivate const\n  Secret: integer := 42;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("App.Lib.Secret"),
        "error should mention the qualified private const, got: {stderr_output}"
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
        "program Main;\nuses App.Lib;\nbegin\n  App.Lib.DoSecret()\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nprivate procedure DoSecret();\nbegin\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Private unit members are not visible outside their unit"),
        "error should hint at private visibility, got: {stderr_output}"
    );
}

#[test]
fn public_const_imported_from_unit() {
    let cwd = create_temp_dir("vis-public-const");
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
        "program Main;\nuses App.Config, Std.Console;\nbegin\n  WriteLn(MaxSize)\nend.\n",
    );
    write_text(
        &cwd.join("src/config.fpas"),
        "unit App.Config;\n\nconst\n  MaxSize: integer := 1024;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "1024\n");
}

#[test]
fn explicit_public_function_is_exported() {
    let cwd = create_temp_dir("vis-explicit-public");
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
        "unit App.Lib;\n\npublic function GetValue(): integer;\nbegin\n  return 88\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "88\n");
}
