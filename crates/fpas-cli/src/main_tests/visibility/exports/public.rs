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
