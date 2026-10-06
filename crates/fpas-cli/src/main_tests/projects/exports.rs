use super::*;

#[test]
fn check_cli_rejects_non_exported_library_unit() {
    let cwd = create_temp_dir("check-lib-export-violation");
    let lib_dir = cwd.join("mylib");
    let app_dir = cwd.join("app");
    let lib_project = lib_dir.join("mylib.fpasprj");
    let app_project = app_dir.join("app.fpasprj");

    write_text(
        &lib_project,
        r#"[project]
name = "mylib"
kind = "library"

[exports]
units = ["MyLib.Core"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(
        &lib_dir.join("src/core.fpas"),
        "unit MyLib.Core;\nuses MyLib.Internal;\nfunction Double(X: integer): integer;\nbegin\n  return Scale(X);\nend function;\nend unit;\n",
    );
    write_text(
        &lib_dir.join("src/internal.fpas"),
        "unit MyLib.Internal;\nfunction Scale(X: integer): integer;\nbegin\n  return X + X;\nend function;\nend unit;\n",
    );

    let lib_dep = toml_path(&lib_project);
    write_text(
        &app_project,
        &format!(
            r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[dependencies]
projects = ["{lib_dep}"]

[sources]
include = ["src/**/*.fpas"]
"#
        ),
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program App;\nuses MyLib.Internal, Std.Console;\nbegin\n  WriteLn(Scale(3));\nend.\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[
            String::from("check"),
            app_project.to_string_lossy().to_string(),
        ],
        &app_dir,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1, "stderr: {stderr_output}");
    assert!(
        stderr_output.contains("not exported"),
        "stderr: {stderr_output}"
    );
}

#[test]
fn run_cli_executes_library_deps_example_with_exports() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");
    let app_project = repo_root.join("examples/pascal/library-deps/app/app.fpasprj");

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&app_project, repo_root);

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "42\n");
    assert!(stderr_output.is_empty());
}

fn toml_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[test]
fn run_cli_accepts_exported_functions_with_typed_task_parameters() {
    let cwd = create_temp_dir("run-lib-typed-task-parameter");
    let lib_dir = cwd.join("sessions");
    let app_dir = cwd.join("app");
    let lib_project = lib_dir.join("sessions.fpasprj");
    let app_project = app_dir.join("app.fpasprj");

    write_text(
        &lib_project,
        r#"[project]
name = "sessions"
kind = "library"

[exports]
units = ["Sessions.Outcome"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(
        &lib_dir.join("src/outcome.fpas"),
        "unit Sessions.Outcome;\nuses Std.Tasks;\npublic function Describe(Session: task of result of boolean, string): string;\nbegin\n  case Wait(Session) of\n    when Ok(Done): begin return 'ok'; end;\n    when Error(Message): begin return Message; end;\n  end case;\nend function;\nend unit;\n",
    );

    let lib_dep = toml_path(&lib_project);
    write_text(
        &app_project,
        &format!(
            r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[dependencies]
projects = ["{lib_dep}"]

[sources]
include = ["src/**/*.fpas"]
"#
        ),
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program App;\nuses Sessions.Outcome, Std.Console, Std.Tasks;\nfunction Session(): result of boolean, string;\nbegin\n  return Error('session failed');\nend function;\nbegin\n  var Job: task := go Session();\n  WriteLn(Describe(Job));\nend.\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&app_project, &app_dir);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "{stderr_output}");
    assert_eq!(stdout_output, "session failed\n");
}
