use super::*;

#[test]
fn run_cli_executes_sole_program_from_workspace_in_cwd() {
    let cwd = create_temp_dir("run-workspace-single-program");
    let workspace_file = cwd.join("suite.fpasworkspace");
    let lib_project = cwd.join("libs/greet/greet.fpasprj");
    let app_project = cwd.join("apps/hello/hello.fpasprj");

    write_text(
        &workspace_file,
        r#"[workspace]
name = "suite"
members = ["libs/greet/greet.fpasprj", "apps/hello/hello.fpasprj"]
"#,
    );
    write_text(
        &lib_project,
        r#"[project]
name = "greet"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(
        &lib_project.parent().unwrap().join("src/greet.fpas"),
        "unit Demo.Greet;\nconst Message: string := 'from lib';\n",
    );

    write_text(
        &app_project,
        r#"[project]
name = "hello"
kind = "program"
main = "src/main.fpas"

[dependencies]
workspace = ["greet"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(
        &app_project.parent().unwrap().join("src/main.fpas"),
        "program Hello;\nuses Demo.Greet, Std.Console;\nbegin\n  WriteLn(Message)\nend.\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_args_and_capture_output(&[String::from("run")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "from lib\n");
    assert!(stderr_output.is_empty());
}

#[test]
fn run_cli_errors_when_workspace_has_multiple_programs() {
    let cwd = create_temp_dir("run-workspace-many-programs");
    let workspace_file = cwd.join("suite.fpasworkspace");

    write_text(
        &workspace_file,
        r#"[workspace]
name = "suite"
members = ["a.fpasprj", "b.fpasprj"]
"#,
    );
    for name in ["a", "b"] {
        let project = cwd.join(format!("{name}.fpasprj"));
        write_text(
            &project,
            &format!(
                r#"[project]
name = "{name}"
kind = "program"
main = "main.fpas"

[sources]
include = ["main.fpas"]
"#
            ),
        );
        write_text(
            &cwd.join(format!("{name}.fpas")),
            "program P;\nbegin\nend.\n",
        );
    }

    let (exit_code, _, stderr_output) =
        support::run_cli_args_and_capture_output(&[String::from("run")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("multiple `program` projects"),
        "stderr: {stderr_output}"
    );
}

#[test]
fn run_cli_errors_when_workspace_has_no_program() {
    let cwd = create_temp_dir("run-workspace-no-program");
    let workspace_file = cwd.join("suite.fpasworkspace");

    write_text(
        &workspace_file,
        r#"[workspace]
name = "suite"
members = ["lib.fpasprj"]
"#,
    );
    write_text(
        &cwd.join("lib.fpasprj"),
        r#"[project]
name = "lib"
kind = "library"

[sources]
include = ["lib.fpas"]
"#,
    );
    write_text(&cwd.join("lib.fpas"), "unit L.Core;\n");

    let (exit_code, _, stderr_output) =
        support::run_cli_args_and_capture_output(&[String::from("run")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("No `program` projects found"),
        "stderr: {stderr_output}"
    );
}

#[test]
fn run_cli_errors_when_multiple_workspace_files_in_cwd() {
    let cwd = create_temp_dir("run-multiple-workspaces");
    for name in ["a.fpasworkspace", "b.fpasworkspace"] {
        write_text(
            &cwd.join(name),
            r#"[workspace]
name = "suite"
members = []
"#,
        );
    }

    let (exit_code, _, stderr_output) =
        support::run_cli_args_and_capture_output(&[String::from("run")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("multiple `.fpasworkspace` files"),
        "stderr: {stderr_output}"
    );
}
