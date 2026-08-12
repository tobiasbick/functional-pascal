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

#[test]
fn debug_jsonl_expression_set_commits_before_continuation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root.join("tests/debugger/fixtures/expression_mutation.fpas");
    let cwd = create_temp_dir("debug-expression-mutation");
    let commands = cwd.join("commands.jsonl");
    write_text(
        &commands,
        "{\"type\":\"request\",\"id\":1,\"command\":\"initialize\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":2,\"command\":\"launch\",\"arguments\":{\"stop_on_entry\":true}}\n{\"type\":\"request\",\"id\":3,\"command\":\"step_into\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":4,\"command\":\"step_into\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":5,\"command\":\"expression.set\",\"arguments\":{\"target\":\"GlobalValue\",\"expression\":\"99\"}}\n{\"type\":\"request\",\"id\":6,\"command\":\"continue\",\"arguments\":{}}\n",
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
    assert!(records.iter().all(serde_json::Value::is_object));
    assert!(
        records.iter().any(|record| {
            record["command"] == "expression.set"
                && record["success"] == true
                && record["body"]["result"] == "99"
        }),
        "{records:?}"
    );
    assert!(
        records
            .iter()
            .any(|record| { record["event"] == "output" && record["body"]["text"] == "99\n" })
    );
    assert!(records.iter().any(|record| record["event"] == "terminated"));
}

#[test]
fn debug_jsonl_dictionary_mutations_commit_before_continuation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root.join("tests/debugger/fixtures/dictionary_mutation.fpas");
    let cwd = create_temp_dir("debug-dictionary-mutation");
    let commands = cwd.join("commands.jsonl");
    write_text(
        &commands,
        "{\"type\":\"request\",\"id\":1,\"command\":\"initialize\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":2,\"command\":\"launch\",\"arguments\":{\"stop_on_entry\":true}}\n{\"type\":\"request\",\"id\":3,\"command\":\"step_into\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":4,\"command\":\"step_into\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":5,\"command\":\"step_into\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":6,\"command\":\"step_into\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":7,\"command\":\"step_into\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":8,\"command\":\"step_into\",\"arguments\":{}}\n{\"type\":\"request\",\"id\":9,\"command\":\"dictionary.insert\",\"arguments\":{\"target\":\"Scores\",\"key\":\"'Bob'\",\"expression\":\"3\"}}\n{\"type\":\"request\",\"id\":10,\"command\":\"dictionary.remove\",\"arguments\":{\"target\":\"Scores\",\"key\":\"'Ada'\"}}\n{\"type\":\"request\",\"id\":11,\"command\":\"dictionary.replace_key\",\"arguments\":{\"target\":\"Scores\",\"key\":\"'Grace'\",\"new_key\":\"'Hopper'\"}}\n{\"type\":\"request\",\"id\":12,\"command\":\"continue\",\"arguments\":{}}\n",
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
    for command in [
        "dictionary.insert",
        "dictionary.remove",
        "dictionary.replace_key",
    ] {
        assert!(
            records
                .iter()
                .any(|record| { record["command"] == command && record["success"] == true }),
            "missing successful {command}: {records:?}"
        );
    }
    let output = records
        .iter()
        .filter(|record| record["event"] == "output")
        .filter_map(|record| record["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "2\n3\n");
    assert!(records.iter().any(|record| record["event"] == "terminated"));
}
