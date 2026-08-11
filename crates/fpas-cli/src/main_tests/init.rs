use super::*;

use crate::cli_input::{InitKind, InitReportFormat};
use crate::main_tests::support::run_cli_args_and_capture_output;

#[test]
fn init_help_is_layered_and_includes_examples() {
    let cwd = create_temp_dir("init-help");

    for (args, expected, excluded) in [
        (
            vec!["init", "--help"],
            "fpas init project <name>",
            "--unit <name>",
        ),
        (
            vec!["init", "project", "--help"],
            "fpas init project <name> [--path",
            "fpas init library <name>",
        ),
        (
            vec!["init", "library", "--help"],
            "--unit <name>",
            "fpas init workspace <name>",
        ),
        (
            vec!["init", "workspace", "--help"],
            "program and a consumed library",
            "--unit <name>",
        ),
    ] {
        let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
        let (code, stdout, stderr) = run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(code, 0, "help must succeed: {stderr}");
        assert!(stdout.contains(expected), "unexpected help: {stdout}");
        assert!(!stdout.contains(excluded), "help is not focused: {stdout}");
        assert!(stdout.contains("Examples:"));
        assert!(stderr.is_empty());
    }

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn init_parser_resolves_non_interactive_options() {
    let cwd = create_temp_dir("init-options");
    let result = resolve_cli_config(
        &[
            "init",
            "library",
            "greet",
            "--path",
            "libs/greet",
            "--unit",
            "Demo.Greet",
            "--dry-run",
            "--report",
            "json",
        ]
        .map(str::to_string),
        &cwd,
    );

    match result {
        Ok(ResolvedCli::Init(config)) => {
            assert_eq!(config.kind, InitKind::Library);
            assert_eq!(config.name, "greet");
            assert_eq!(config.root, cwd.join("libs/greet"));
            assert_eq!(config.library_unit.as_deref(), Some("Demo.Greet"));
            assert!(config.dry_run);
            assert_eq!(config.report, Some(InitReportFormat::Json));
        }
        other => panic!("expected init config, got {other:?}"),
    }

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn init_project_creates_formatted_checkable_scaffold() {
    let cwd = create_temp_dir("init-project");
    let (code, stdout, stderr) = run_init(&cwd, &["project", "my-app"]);

    assert_eq!(code, 0, "init failed: {stderr}");
    assert!(stdout.contains("status: created"));
    assert!(cwd.join("my-app/.gitignore").is_file());
    assert!(cwd.join("my-app/my-app.fpasprj").is_file());
    assert_eq!(
        fs::read_to_string(cwd.join("my-app/src/main.fpas")).expect("read generated program"),
        "program MyApp;\n\nuses Std.Console;\n\nbegin\n  WriteLn('Hello from my-app')\nend.\n"
    );
    assert_cli_succeeds(&cwd, &["check", "my-app/my-app.fpasprj"]);
    assert_cli_succeeds(&cwd, &["fmt", "--check", "my-app/my-app.fpasprj"]);

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn init_library_uses_explicit_exported_unit_and_is_checkable() {
    let cwd = create_temp_dir("init-library");
    let (code, _, stderr) = run_init(&cwd, &["library", "greet", "--unit", "Demo.Greet"]);

    assert_eq!(code, 0, "init failed: {stderr}");
    let manifest = fs::read_to_string(cwd.join("greet/greet.fpasprj"))
        .expect("read generated library manifest");
    assert!(manifest.contains("kind = \"library\""));
    assert!(manifest.contains("units = [\"Demo.Greet\"]"));
    assert!(cwd.join("greet/src/greet.fpas").is_file());
    assert_cli_succeeds(&cwd, &["check", "greet/greet.fpasprj"]);
    assert_cli_succeeds(&cwd, &["fmt", "--check", "greet/greet.fpasprj"]);

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn init_workspace_creates_linked_program_and_library() {
    let cwd = create_temp_dir("init-workspace");
    let (code, _, stderr) = run_init(&cwd, &["workspace", "acme-suite"]);

    assert_eq!(code, 0, "init failed: {stderr}");
    let workspace = cwd.join("acme-suite/acme-suite.fpasworkspace");
    assert!(workspace.is_file());
    assert!(
        cwd.join("acme-suite/libs/acme-suite-core/acme-suite-core.fpasprj")
            .is_file()
    );
    let app_manifest =
        fs::read_to_string(cwd.join("acme-suite/apps/acme-suite/acme-suite-app.fpasprj"))
            .expect("read generated app manifest");
    assert!(app_manifest.contains("workspace = [\"acme-suite-core\"]"));
    assert_cli_succeeds(&cwd, &["check", path_text(&workspace).as_str()]);
    assert_cli_succeeds(&cwd, &["fmt", "--check", path_text(&workspace).as_str()]);

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn init_dry_run_json_is_machine_readable_and_writes_nothing() {
    let cwd = create_temp_dir("init-dry-run");
    let (code, stdout, stderr) =
        run_init(&cwd, &["project", "hello", "--dry-run", "--report", "json"]);

    assert_eq!(code, 0, "dry-run failed: {stderr}");
    assert!(stderr.is_empty());
    let report: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON report");
    assert_eq!(report["status"], "planned");
    assert_eq!(report["kind"], "project");
    assert_eq!(report["manifest"], "hello/hello.fpasprj");
    assert!(!cwd.join("hello").exists());

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn init_is_idempotent_but_never_overwrites_conflicts() {
    let cwd = create_temp_dir("init-idempotent");
    let (first_code, _, first_stderr) = run_init(&cwd, &["project", "hello"]);
    assert_eq!(first_code, 0, "first init failed: {first_stderr}");

    let (second_code, second_stdout, second_stderr) = run_init(&cwd, &["project", "hello"]);
    assert_eq!(second_code, 0, "second init failed: {second_stderr}");
    assert!(second_stdout.contains("status: unchanged"));

    write_text(&cwd.join("conflict/conflict.fpasprj"), "user content\n");
    let (conflict_code, _, conflict_stderr) = run_init(&cwd, &["project", "conflict"]);
    assert_eq!(conflict_code, 1);
    assert!(conflict_stderr.contains("existing files differ"));
    assert_eq!(
        fs::read_to_string(cwd.join("conflict/conflict.fpasprj"))
            .expect("conflicting file remains"),
        "user content\n"
    );
    assert!(!cwd.join("conflict/src/main.fpas").exists());
    assert!(!cwd.join("conflict/.gitignore").exists());

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn init_rejects_missing_invalid_and_ambiguous_arguments() {
    let cwd = create_temp_dir("init-invalid");
    for (args, expected) in [
        (vec!["init", "project"], "Missing name"),
        (vec!["init", "unknown", "demo"], "Unknown scaffold kind"),
        (
            vec!["init", "project", "two--words"],
            "Invalid scaffold name",
        ),
        (vec!["init", "project", "123app"], "Invalid scaffold name"),
        (
            vec!["init", "project", "program"],
            "derives the reserved Functional Pascal identifier",
        ),
        (
            vec!["init", "library", "demo", "--unit", "Bad-Unit"],
            "Invalid Functional Pascal unit name",
        ),
        (
            vec!["init", "library", "demo", "--unit", "Demo.Program"],
            "Invalid Functional Pascal unit name",
        ),
        (
            vec!["init", "project", "demo", "--path", "--dry-run"],
            "is an option and cannot be the value",
        ),
        (
            vec!["init", "project", "demo", "--unit", "Demo.Core"],
            "Unknown option `--unit`",
        ),
    ] {
        let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
        let (code, stdout, stderr) = run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(code, 1, "invalid invocation succeeded: {args:?}");
        assert!(stdout.is_empty());
        assert!(stderr.contains(expected), "unexpected error: {stderr}");
    }

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

fn run_init(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let args = std::iter::once("init")
        .chain(args.iter().copied())
        .map(str::to_string)
        .collect::<Vec<_>>();
    run_cli_args_and_capture_output(&args, cwd)
}

fn assert_cli_succeeds(cwd: &Path, args: &[&str]) {
    let args = args
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let (code, _, stderr) = run_cli_args_and_capture_output(&args, cwd);
    assert_eq!(code, 0, "CLI invocation failed: {stderr}");
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
