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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(Secret())\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nfunction Secret(): integer;\nbegin\n  return 42\nend;\n",
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
        "unit App.Lib;\n\nfunction Secret(): integer;\nbegin\n  return 42\nend;\n",
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
        "unit App.Lib;\n\nconst\n  Secret: integer := 42;\n",
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
        "program Main;\nuses App.Lib;\nbegin\n  DoSecret()\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nprocedure DoSecret();\nbegin\nend;\n",
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
        "\
program Main;
uses App.Lib;
begin
  var P: SecretPoint := record X := 1; Y := 2; end;
end.
",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "\
unit App.Lib;

type
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
        "\
program Main;
uses App.Lib;
begin
  var P: App.Lib.SecretPoint := record X := 1; Y := 2; end;
end.
",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "\
unit App.Lib;

type
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(Secret)\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nvar\n  Secret: integer := 42;\n",
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(Counter)\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nmutable var\n  Counter: integer := 0;\n",
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(App.Lib.Secret)\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nvar\n  Secret: integer := 42;\n",
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
        "program Main;\nuses App.Lib, Std.Console;\nbegin\n  WriteLn(App.Lib.Secret)\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nconst\n  Secret: integer := 42;\n",
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
        "program Main;\nuses App.Lib;\nbegin\n  App.Lib.DoSecret()\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\n\nprocedure DoSecret();\nbegin\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Private unit members are not visible outside their unit"),
        "error should hint at private visibility, got: {stderr_output}"
    );
}
