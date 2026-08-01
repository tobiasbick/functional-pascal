use super::*;

#[test]
fn build_cli_creates_and_then_reuses_named_program_artifact() {
    let cwd = create_temp_dir("build-program-artifact");
    let project_file = cwd.join("manifest.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "hello-app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&cwd.join("src/main.fpas"), "program Hello;\nbegin\nend.\n");

    let args = [
        String::from("build"),
        project_file.to_string_lossy().into_owned(),
    ];
    let (cold_code, cold_stdout, cold_stderr) =
        support::run_cli_args_and_capture_output(&args, &cwd);
    let (warm_code, warm_stdout, warm_stderr) =
        support::run_cli_args_and_capture_output(&args, &cwd);
    let artifact_exists = cwd.join("hello-app.fpascp").is_file();
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(cold_code, 0, "stderr: {cold_stderr}");
    assert!(cold_stdout.contains("Built program `hello-app`"));
    assert_eq!(warm_code, 0, "stderr: {warm_stderr}");
    assert!(warm_stdout.contains("Reused program `hello-app`"));
    assert!(artifact_exists);
}

#[test]
fn build_cli_reports_main_diagnostics_against_the_source_path() {
    let cwd = create_temp_dir("build-main-diagnostic-path");
    let project_file = cwd.join("manifest.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "broken-app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    let main = cwd.join("src/main.fpas");
    write_text(&main, "program Broken;\nbegin\n  MissingCall()\nend.\n");

    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("build"),
            project_file.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr.contains("src/main.fpas`: 3:3: error[F2003]"),
        "stderr: {stderr}"
    );
    assert!(!stderr.contains("manifest.fpasprj`: 3:3"));
}

#[test]
fn build_cli_builds_library_unit_sidecars_without_program_artifact() {
    let cwd = create_temp_dir("build-library-artifact");
    let project_file = cwd.join("library.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "math-library"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(
        &cwd.join("src/math.fpas"),
        "unit Demo.Math;\npublic const Answer: integer := 42;\n",
    );

    let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("build"),
            project_file.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    let sidecar_exists = cwd.join("src/math.fpascu").is_file();
    let program_artifact_exists = cwd.join("math-library.fpascp").exists();
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr}");
    assert!(stdout.contains("Built library `math-library`"));
    assert!(sidecar_exists);
    assert!(!program_artifact_exists);
}

#[test]
fn build_cli_workspace_processes_library_and_program_members() {
    let cwd = create_temp_dir("build-workspace");
    let workspace_file = cwd.join("suite.fpasworkspace");
    let library_project = cwd.join("libs/math/math.fpasprj");
    let program_project = cwd.join("apps/hello/hello.fpasprj");
    write_text(
        &workspace_file,
        r#"[workspace]
name = "suite"
members = ["libs/math/math.fpasprj", "apps/hello/hello.fpasprj"]
"#,
    );
    write_text(
        &library_project,
        r#"[project]
name = "math"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(
        &cwd.join("libs/math/src/math.fpas"),
        "unit Demo.Math;\npublic function Answer(): integer;\nbegin\n  return 42\nend;\n",
    );
    write_text(
        &program_project,
        r#"[project]
name = "hello"
kind = "program"
main = "src/main.fpas"

[dependencies]
workspace = ["math"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(
        &cwd.join("apps/hello/src/main.fpas"),
        "program Hello;\nuses Demo.Math;\nbegin\n  Answer()\nend.\n",
    );

    let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("build"),
            workspace_file.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    let unit_exists = cwd.join("libs/math/src/math.fpascu").is_file();
    let program_exists = cwd.join("apps/hello/hello.fpascp").is_file();
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr}");
    assert!(stdout.contains("Built library `math`"));
    assert!(stdout.contains("Built program `hello`"));
    assert!(unit_exists);
    assert!(program_exists);
}

#[test]
fn build_cli_validates_test_projects_without_shared_program_artifact() {
    let cwd = create_temp_dir("build-test-project");
    let project_file = cwd.join("tests.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "demo-tests"
kind = "test"

[sources]
include = ["**/*.fpas"]
"#,
    );
    write_text(
        &cwd.join("support.fpas"),
        "unit Demo.Support;\npublic function Answer(): integer;\nbegin\n  return 42\nend;\n",
    );
    write_text(
        &cwd.join("answer_test.fpas"),
        "program AnswerTest;\nuses Demo.Support;\nbegin\n  Answer()\nend.\n",
    );

    let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("build"),
            project_file.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    let sidecar_exists = cwd.join("support.fpascu").is_file();
    let shared_program_exists = cwd.join("demo-tests.fpascp").exists();
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr}");
    assert!(stdout.contains("Built test project `demo-tests`"));
    assert!(sidecar_exists);
    assert!(!shared_program_exists);
}

#[test]
fn build_cli_rejects_source_files_with_an_actionable_error() {
    let cwd = create_temp_dir("build-source-rejected");
    let source = cwd.join("main.fpas");
    write_text(&source, "program Main;\nbegin\nend.\n");

    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &[String::from("build"), source.to_string_lossy().into_owned()],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr.contains("Expected a `.fpasprj` or `.fpasworkspace` file"));
}

#[test]
fn build_cli_rejects_project_names_that_escape_the_project_directory() {
    let cwd = create_temp_dir("build-invalid-project-name");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "../outside"
kind = "program"
main = "main.fpas"

[sources]
include = ["main.fpas"]
"#,
    );
    write_text(&cwd.join("main.fpas"), "program Main;\nbegin\nend.\n");

    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("build"),
            project_file.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    let escaped_artifact_exists = cwd
        .parent()
        .is_some_and(|parent| parent.join("outside.fpascp").exists());
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr.contains("cannot be used as an artifact filename"));
    assert!(!escaped_artifact_exists);
}

#[test]
fn build_executable_rejects_non_program_project_before_runner_lookup() {
    let cwd = create_temp_dir("build-native-library");
    let project_file = cwd.join("library.fpasprj");
    support::write_library_project_file(&project_file, &["src/**/*.fpas"]);
    write_text(&cwd.join("src/lib.fpas"), "unit Demo.Lib;\n");

    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("build"),
            String::from("--executable"),
            project_file.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr.contains("require a `program` project"));
}

#[test]
fn build_executable_rejects_invalid_application_name_before_building() {
    let cwd = create_temp_dir("build-native-invalid-name");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "main.fpas", &["main.fpas"]);
    write_text(&cwd.join("main.fpas"), "program Main;\nbegin\nend.\n");

    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("build"),
            String::from("--executable"),
            String::from("--name"),
            String::from("../escape"),
            project_file.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    let image_exists = cwd.join("app.fpascp").exists();
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr.contains("cannot be used as an executable filename"));
    assert!(!image_exists);
}

#[test]
fn build_executable_rejects_workspace_with_multiple_programs() {
    let cwd = create_temp_dir("build-native-multiple-programs");
    let workspace = cwd.join("suite.fpasworkspace");
    write_text(
        &workspace,
        r#"[workspace]
name = "suite"
members = ["a.fpasprj", "b.fpasprj"]
"#,
    );
    for name in ["a", "b"] {
        write_text(
            &cwd.join(format!("{name}.fpasprj")),
            &format!(
                r#"[project]
name = "{name}"
kind = "program"
main = "{name}.fpas"

[sources]
include = ["{name}.fpas"]
"#
            ),
        );
        write_text(
            &cwd.join(format!("{name}.fpas")),
            "program Main;\nbegin\nend.\n",
        );
    }

    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("build"),
            String::from("--executable"),
            workspace.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr.contains("multiple `program` projects"));
}
