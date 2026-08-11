use super::*;

fn debug_args(source: &Path, commands: &Path) -> Vec<String> {
    vec![
        "debug".into(),
        source.to_string_lossy().into_owned(),
        "--protocol".into(),
        "jsonl".into(),
        "--commands".into(),
        commands.to_string_lossy().into_owned(),
    ]
}

#[test]
fn debug_jsonl_script_emits_only_json_records() {
    let cwd = create_temp_dir("debug-jsonl");
    let source = cwd.join("main.fpas");
    let commands = cwd.join("commands.jsonl");
    write_text(&source, "program Main; begin var X: integer := 1 end.\n");
    write_text(
        &commands,
        "{\"type\":\"request\",\"id\":1,\"command\":\"initialize\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":2,\"command\":\"launch\",\"arguments\":{\"stop_on_entry\":false}}\n",
    );
    let (exit, stdout, stderr) =
        support::run_cli_args_and_capture_output(&debug_args(&source, &commands), &cwd);
    fs::remove_dir_all(&cwd).expect("remove debugger temp directory");
    assert_eq!(exit, 0, "stderr: {stderr}");
    assert!(stderr.is_empty());
    let records = stdout
        .lines()
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line).expect("protocol stdout must be JSONL")
        })
        .collect::<Vec<_>>();
    assert!(records.iter().any(|record| record["event"] == "terminated"));
}

#[test]
fn debug_compile_failure_stays_off_protocol_stdout() {
    let cwd = create_temp_dir("debug-compile-failure");
    let source = cwd.join("broken.fpas");
    let commands = cwd.join("commands.jsonl");
    write_text(&source, "program Broken; begin Missing() end.\n");
    write_text(
        &commands,
        "{\"type\":\"request\",\"id\":1,\"command\":\"initialize\",\"arguments\":{}}\n",
    );
    let (exit, stdout, stderr) =
        support::run_cli_args_and_capture_output(&debug_args(&source, &commands), &cwd);
    fs::remove_dir_all(&cwd).expect("remove debugger temp directory");
    assert_eq!(exit, 1);
    assert!(stdout.is_empty());
    assert!(stderr.contains("Unknown procedure `Missing`"));
}

#[test]
fn debug_output_limit_is_a_stable_protocol_error() {
    let cwd = create_temp_dir("debug-output-limit");
    let source = cwd.join("main.fpas");
    let commands = cwd.join("commands.jsonl");
    write_text(
        &source,
        "program Main; uses Std.Console; begin WriteLn('too much output') end.\n",
    );
    write_text(
        &commands,
        "{\"type\":\"request\",\"id\":1,\"command\":\"initialize\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":2,\"command\":\"launch\",\"arguments\":{\"stop_on_entry\":false}}\n",
    );
    let mut args = debug_args(&source, &commands);
    args.extend(["--output-limit".into(), "1".into()]);
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    fs::remove_dir_all(&cwd).expect("remove debugger temp directory");
    assert_eq!(exit, 0, "stderr: {stderr}");
    assert!(
        stdout
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .any(|record| record["body"]["code"] == "output_limit")
    );
}

#[test]
fn debug_accepts_program_projects_and_workspaces() {
    let cwd = create_temp_dir("debug-project-inputs");
    let project = cwd.join("app/app.fpasprj");
    let workspace = cwd.join("suite.fpasworkspace");
    let source = cwd.join("app/src/main.fpas");
    let commands = cwd.join("commands.jsonl");
    crate::test_support::write_program_fpasprj(&project, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(&source, "program Main; begin end.\n");
    write_text(
        &workspace,
        "[workspace]\nname = \"suite\"\nmembers = [\"app/app.fpasprj\"]\n",
    );
    write_text(
        &commands,
        "{\"type\":\"request\",\"id\":1,\"command\":\"initialize\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":2,\"command\":\"launch\",\"arguments\":{\"stop_on_entry\":false}}\n",
    );
    for target in [&project, &workspace] {
        let (exit, stdout, stderr) =
            support::run_cli_args_and_capture_output(&debug_args(target, &commands), &cwd);
        assert_eq!(exit, 0, "target: {}; stderr: {stderr}", target.display());
        assert!(
            stdout
                .lines()
                .all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok())
        );
    }
    fs::remove_dir_all(&cwd).expect("remove debugger temp directory");
}

#[test]
fn debug_runs_reachable_task_spawning_and_emits_lifecycle_events() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root.join("tests/debugger/fixtures/task_debugging.fpas");
    let cwd = create_temp_dir("debug-tasks");
    let commands = cwd.join("commands.jsonl");
    write_text(
        &commands,
        "{\"type\":\"request\",\"id\":1,\"command\":\"initialize\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":2,\"command\":\"launch\",\"arguments\":{\"stop_on_entry\":false}}\n",
    );
    let (exit, stdout, stderr) =
        support::run_cli_args_and_capture_output(&debug_args(&source, &commands), &root);
    fs::remove_dir_all(&cwd).expect("remove debugger temp directory");
    assert_eq!(exit, 0, "stderr: {stderr}");
    assert!(stderr.is_empty());
    let records = stdout
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("debugger JSONL"))
        .collect::<Vec<_>>();
    assert!(
        records
            .iter()
            .any(|record| { record["event"] == "task" && record["body"]["reason"] == "started" })
    );
    assert!(
        records
            .iter()
            .any(|record| { record["event"] == "task" && record["body"]["reason"] == "exited" })
    );
    assert!(
        records
            .iter()
            .any(|record| { record["event"] == "output" && record["body"]["text"] == "42\n" })
    );
    assert!(records.iter().any(|record| record["event"] == "terminated"));
}
