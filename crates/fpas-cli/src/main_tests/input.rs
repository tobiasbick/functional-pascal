use super::support::run_cli_args_and_capture_output;
use super::{ResolvedCli, resolve_cli_config, *};

#[test]
fn resolve_cli_input_uses_explicit_source_file() {
    let cwd = create_temp_dir("source");
    let result = resolve_cli_input(&[String::from("src/main.fpas")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(result, Ok(CliInput::SourceFile(cwd.join("src/main.fpas"))));
}

#[test]
fn resolve_cli_input_uses_explicit_project_file() {
    let cwd = create_temp_dir("project");
    let result = resolve_cli_input(&[String::from("my-app.fpasprj")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(
        result,
        Ok(CliInput::ProjectFile(cwd.join("my-app.fpasprj")))
    );
}

#[test]
fn resolve_cli_input_rejects_unknown_extension() {
    let cwd = create_temp_dir("unknown-ext");
    let result = resolve_cli_input(&[String::from("project.toml")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("unknown extension must fail");
    assert!(error.contains("Unsupported input"));
    assert!(error.contains(".fpas"));
    assert!(error.contains(".fpasprj"));
}

#[test]
fn resolve_cli_input_discovers_workspace_program_when_no_args_are_given() {
    let cwd = create_temp_dir("discover-workspace-run");
    let workspace_file = cwd.join("suite.fpasworkspace");
    write_text(
        &workspace_file,
        r#"[workspace]
name = "suite"
members = ["lib.fpasprj", "app.fpasprj"]
"#,
    );
    write_file(&cwd.join("lib.fpasprj"));
    write_file(&cwd.join("app.fpasprj"));
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
    write_text(
        &cwd.join("app.fpasprj"),
        r#"[project]
name = "app"
kind = "program"
main = "main.fpas"

[sources]
include = ["main.fpas"]
"#,
    );
    write_text(&cwd.join("main.fpas"), "program App;\nbegin\nend.\n");

    let result = resolve_cli_input(&[], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(result, Ok(CliInput::ProjectFile(cwd.join("app.fpasprj"))));
}

#[test]
fn resolve_cli_input_errors_when_multiple_workspace_files_in_cwd() {
    let cwd = create_temp_dir("discover-multiple-workspaces");
    for name in ["a.fpasworkspace", "b.fpasworkspace"] {
        write_text(
            &cwd.join(name),
            r#"[workspace]
name = "suite"
members = []
"#,
        );
    }

    let run_result = resolve_cli_input(&[], &cwd);
    let check_result = resolve_cli_config(&[String::from("check")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let run_error = run_result.expect_err("run must fail with multiple workspaces");
    assert!(run_error.contains("multiple `.fpasworkspace` files"));

    let check_error = match check_result {
        Ok(ResolvedCli::Check(_)) => panic!("check must fail with multiple workspaces"),
        Ok(_) => panic!("unexpected resolved cli"),
        Err(message) => message,
    };
    assert!(check_error.contains("multiple `.fpasworkspace` files"));
}

#[test]
fn resolve_cli_input_discovers_project_file_when_no_args_are_given() {
    let cwd = create_temp_dir("discover-one");
    let project_path = cwd.join("demo.fpasprj");
    write_file(&project_path);

    let result = resolve_cli_input(&[], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(result, Ok(CliInput::ProjectFile(project_path)));
}

#[test]
fn resolve_cli_input_fails_when_no_project_file_exists() {
    let cwd = create_temp_dir("discover-none");
    let result = resolve_cli_input(&[], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("missing project file must fail");
    assert!(error.contains("No `.fpasprj` file found"));
}

#[test]
fn resolve_cli_input_fails_when_multiple_project_files_exist() {
    let cwd = create_temp_dir("discover-many");
    write_file(&cwd.join("a.fpasprj"));
    write_file(&cwd.join("b.fpasprj"));

    let result = resolve_cli_input(&[], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("multiple project files must fail");
    assert!(error.contains("Found multiple `.fpasprj` files"));
    assert!(error.contains("a.fpasprj"));
    assert!(error.contains("b.fpasprj"));
}

#[test]
fn resolve_cli_input_handles_case_insensitive_extensions() {
    let cwd = create_temp_dir("case-ext");
    let result_fpas = resolve_cli_input(&[String::from("Main.FPAS")], &cwd);
    let result_prj = resolve_cli_input(&[String::from("app.FPASPRJ")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(result_fpas, Ok(CliInput::SourceFile(cwd.join("Main.FPAS"))));
    assert_eq!(
        result_prj,
        Ok(CliInput::ProjectFile(cwd.join("app.FPASPRJ")))
    );
}

#[test]
fn resolve_cli_input_rejects_more_than_one_argument() {
    let cwd = create_temp_dir("too-many-args");
    let result = resolve_cli_input(&[String::from("a.fpas"), String::from("b.fpas")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("multiple arguments must fail");
    assert!(
        error.starts_with("Usage: fpas [<file.fpas | file.fpasprj>]"),
        "unexpected error: {error}"
    );
}

#[test]
fn resolve_cli_config_splits_program_arguments_after_separator() {
    let cwd = create_temp_dir("program-args");
    let result = resolve_cli_config(
        &[
            String::from("main.fpas"),
            String::from("--"),
            String::from("one"),
            String::from("-two"),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(
        result,
        Ok(ResolvedCli::Run(CliConfig {
            input: CliInput::SourceFile(cwd.join("main.fpas")),
            program_args: vec![String::from("one"), String::from("-two")],
        }))
    );
}

#[test]
fn resolve_cli_config_rejects_define_style_flags() {
    let cwd = create_temp_dir("no-define");
    let result = resolve_cli_config(&[String::from("-DDEBUG"), String::from("main.fpas")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("define flags are not supported");
    assert!(error.contains("Unknown option"));
    assert!(error.contains("-DDEBUG"));
}

#[test]
fn resolve_cli_config_help_and_version_are_exclusive() {
    let cwd = create_temp_dir("flags");
    assert_eq!(
        resolve_cli_config(&[String::from("--help")], &cwd),
        Ok(ResolvedCli::Help)
    );
    assert_eq!(
        resolve_cli_config(&[String::from("-h")], &cwd),
        Ok(ResolvedCli::Help)
    );
    assert_eq!(
        resolve_cli_config(&[String::from("--version")], &cwd),
        Ok(ResolvedCli::Version)
    );
    assert_eq!(
        resolve_cli_config(&[String::from("-V")], &cwd),
        Ok(ResolvedCli::Version)
    );

    let extra = resolve_cli_config(&[String::from("--help"), String::from("x.fpas")], &cwd);
    assert!(extra.is_err());
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn run_cli_help_and_version_exit_zero() {
    let cwd = create_temp_dir("run-help");

    let (code_h, help_text, stderr_h) =
        run_cli_args_and_capture_output(&[String::from("--help")], &cwd);
    assert_eq!(code_h, 0);
    assert!(help_text.contains("Usage:"));
    assert!(stderr_h.is_empty());

    let (code_v, ver, stderr_v) =
        run_cli_args_and_capture_output(&[String::from("--version")], &cwd);
    assert_eq!(code_v, 0);
    assert!(ver.starts_with("fpas "));
    assert!(stderr_v.is_empty());

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn resolve_cli_config_parses_test_report_json_flag() {
    let cwd = create_temp_dir("test-report-json");
    write_text(
        &cwd.join("demo.fpasprj"),
        "[project]\nname = \"demo\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
    );

    let result = resolve_cli_config(
        &[
            String::from("test"),
            String::from("--report"),
            String::from("json"),
            String::from("demo.fpasprj"),
        ],
        &cwd,
    );

    match result {
        Ok(ResolvedCli::Test(config)) => {
            assert_eq!(config.report, Some(crate::TestReportFormat::Json));
        }
        other => panic!("expected test config, got {other:?}"),
    }

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}
