use super::support::run_cli_args_and_capture_output;
use super::{ResolvedCli, resolve_cli_config, *};
use crate::cli_input::HelpTopic;

mod help;
mod value_options;

fn run_args(args: &[&str]) -> Vec<String> {
    std::iter::once("run")
        .chain(args.iter().copied())
        .map(str::to_owned)
        .collect()
}

#[test]
fn resolve_cli_input_uses_explicit_source_file() {
    let cwd = create_temp_dir("source");
    let result = resolve_cli_input(&run_args(&["src/main.fpas"]), &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(result, Ok(CliInput::SourceFile(cwd.join("src/main.fpas"))));
}

#[test]
fn resolve_cli_input_uses_explicit_project_file() {
    let cwd = create_temp_dir("project");
    let result = resolve_cli_input(&run_args(&["my-app.fpasprj"]), &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(
        result,
        Ok(CliInput::ProjectFile(cwd.join("my-app.fpasprj")))
    );
}

#[test]
fn resolve_cli_input_uses_explicit_workspace_and_compiled_program() {
    let cwd = create_temp_dir("run-artifact-inputs");
    let workspace = resolve_cli_input(&run_args(&["suite.fpasworkspace"]), &cwd);
    let compiled = resolve_cli_input(&run_args(&["app.fpascp"]), &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(
        workspace,
        Ok(CliInput::WorkspaceFile(cwd.join("suite.fpasworkspace")))
    );
    assert_eq!(
        compiled,
        Ok(CliInput::CompiledProgramFile(cwd.join("app.fpascp")))
    );
}

#[test]
fn resolve_cli_input_rejects_unknown_extension() {
    let cwd = create_temp_dir("unknown-ext");
    let result = resolve_cli_input(&run_args(&["project.toml"]), &cwd);
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

    let result = resolve_cli_input(&run_args(&[]), &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(
        result,
        Ok(CliInput::WorkspaceFile(cwd.join("suite.fpasworkspace")))
    );
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

    let run_result = resolve_cli_input(&run_args(&[]), &cwd);
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

    let result = resolve_cli_input(&run_args(&[]), &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(result, Ok(CliInput::ProjectFile(project_path)));
}

#[test]
fn resolve_cli_input_fails_when_no_project_file_exists() {
    let cwd = create_temp_dir("discover-none");
    let result = resolve_cli_input(&run_args(&[]), &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("missing project file must fail");
    assert!(error.contains("No `.fpasprj` file found"));
}

#[test]
fn resolve_cli_input_fails_when_multiple_project_files_exist() {
    let cwd = create_temp_dir("discover-many");
    write_file(&cwd.join("a.fpasprj"));
    write_file(&cwd.join("b.fpasprj"));

    let result = resolve_cli_input(&run_args(&[]), &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("multiple project files must fail");
    assert!(error.contains("Found multiple `.fpasprj` files"));
    assert!(error.contains("a.fpasprj"));
    assert!(error.contains("b.fpasprj"));
}

#[test]
fn resolve_cli_input_handles_case_insensitive_extensions() {
    let cwd = create_temp_dir("case-ext");
    let result_fpas = resolve_cli_input(&run_args(&["Main.FPAS"]), &cwd);
    let result_prj = resolve_cli_input(&run_args(&["app.FPASPRJ"]), &cwd);
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
    let result = resolve_cli_input(&run_args(&["a.fpas", "b.fpas"]), &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("multiple arguments must fail");
    assert!(
        error.starts_with(
            "Usage: fpas run [<file.fpas | file.fpasprj | file.fpasworkspace | file.fpascp>]"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn resolve_cli_config_rejects_bare_source_path_without_run_subcommand() {
    let cwd = create_temp_dir("bare-source-path");
    let result = resolve_cli_config(&[String::from("main.fpas")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("bare source path must fail");
    assert!(error.contains("fpas run main.fpas"));
}

#[test]
fn resolve_cli_config_splits_program_arguments_after_separator() {
    let cwd = create_temp_dir("program-args");
    let result = resolve_cli_config(
        &[
            String::from("run"),
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
            standard_library: None,
        }))
    );
}

#[test]
fn resolve_cli_config_rejects_define_style_flags() {
    let cwd = create_temp_dir("no-define");
    let result = resolve_cli_config(
        &[
            String::from("run"),
            String::from("-DDEBUG"),
            String::from("main.fpas"),
        ],
        &cwd,
    );
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
        Ok(ResolvedCli::Help(HelpTopic::General))
    );
    assert_eq!(
        resolve_cli_config(&[String::from("-h")], &cwd),
        Ok(ResolvedCli::Help(HelpTopic::General))
    );
    assert_eq!(
        resolve_cli_config(&[], &cwd),
        Ok(ResolvedCli::Help(HelpTopic::General))
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
fn resolve_cli_config_rejects_program_args_without_run_subcommand() {
    let cwd = create_temp_dir("program-args-without-run");
    let result = resolve_cli_config(&[String::from("--"), String::from("one")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("program args without run must fail");
    assert!(error.contains("require `fpas run`"));
}

#[test]
fn resolve_cli_config_discovers_workspace_for_build() {
    let cwd = create_temp_dir("build-discover-workspace");
    let workspace = cwd.join("suite.fpasworkspace");
    write_text(
        &workspace,
        "[workspace]\nname = \"suite\"\nmembers = [\"app.fpasprj\"]\n",
    );

    let result = resolve_cli_config(&[String::from("build")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    match result {
        Ok(ResolvedCli::Build(config)) => {
            assert_eq!(config.input, CliInput::WorkspaceFile(workspace));
            assert!(!config.executable);
            assert_eq!(config.name, None);
        }
        other => panic!("expected build config, got {other:?}"),
    }
}

#[test]
fn resolve_cli_config_rejects_program_arguments_for_build() {
    let cwd = create_temp_dir("build-program-args");
    let result = resolve_cli_config(
        &[
            String::from("build"),
            String::from("--"),
            String::from("argument"),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("build must reject program arguments");
    assert!(error.contains("`fpas build` does not accept program arguments"));
}

#[test]
fn resolve_cli_config_discovers_project_when_test_has_no_path() {
    let cwd = create_temp_dir("test-discover-one");
    write_text(
        &cwd.join("tests.fpasprj"),
        "[project]\nname = \"tests\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
    );

    let result = resolve_cli_config(&[String::from("test")], &cwd);

    match result {
        Ok(ResolvedCli::Test(config)) => {
            assert_eq!(
                config.input,
                CliInput::ProjectFile(cwd.join("tests.fpasprj"))
            );
        }
        other => panic!("expected test config, got {other:?}"),
    }

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}
