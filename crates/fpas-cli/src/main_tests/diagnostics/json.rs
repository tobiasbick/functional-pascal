//! `--diagnostics json`: every stderr line is one diagnostic record; stdout is unchanged.

use super::*;
use serde_json::Value;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

/// Parses every stderr line as a JSON diagnostic record.
fn records(stderr: &str) -> Vec<Value> {
    stderr
        .lines()
        .map(|line| {
            serde_json::from_str(line)
                .unwrap_or_else(|error| panic!("stderr line is not JSON ({error}): {line}"))
        })
        .collect()
}

fn codes(records: &[Value]) -> Vec<&str> {
    records
        .iter()
        .map(|record| record["code"].as_str().expect("code"))
        .collect()
}

#[test]
fn check_emits_located_compile_records_with_unicode_columns() {
    let cwd = create_temp_dir("json-check");
    let main = cwd.join("main.fpas");
    write_text(
        &main,
        "program Broken;\nuses Std.Console;\nbegin\n  WriteLn('ä😀'); MissingCall()\nend.\n",
    );

    let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(
        &args(&["check", "--diagnostics", "json", &main.to_string_lossy()]),
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert_eq!(stdout, "");
    let records = records(&stderr);
    assert_eq!(codes(&records), ["F2003"]);
    let record = &records[0];
    assert_eq!(record["kind"], "diagnostic");
    assert_eq!(record["phase"], "sema");
    assert!(
        record["source"]
            .as_str()
            .expect("source")
            .ends_with("main.fpas")
    );
    // `ä` and `😀` each count as one scalar column.
    assert_eq!(record["location"]["start"]["line"], 4);
    assert_eq!(record["location"]["start"]["column"], 18);
    assert!(record["location"]["end"].is_object());
}

#[test]
fn build_emits_project_records_without_progress_text() {
    let cwd = create_temp_dir("json-build");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        "[project]\nname = \"app\"\nkind = \"unknown\"\n",
    );

    let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(
        &args(&[
            "build",
            "--diagnostics",
            "json",
            &project_file.to_string_lossy(),
        ]),
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert_eq!(stdout, "");
    let records = records(&stderr);
    assert_eq!(codes(&records), ["F5005"]);
    assert_eq!(records[0]["phase"], "project");
    assert_eq!(records[0]["location"], Value::Null);
    assert!(records[0]["hint"].is_string());
}

#[test]
fn run_keeps_program_stdout_and_exit_status_for_runtime_records() {
    let cwd = create_temp_dir("json-run");
    let main = cwd.join("main.fpas");
    write_text(
        &main,
        "program RuntimeFail;\nuses Std.Console;\nbegin\n  WriteLn('before');\n  panic('boom');\nend.\n",
    );

    let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(
        &args(&["run", "--diagnostics", "json", &main.to_string_lossy()]),
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 2);
    assert_eq!(stdout, "before\n");
    let records = records(&stderr);
    assert_eq!(codes(&records), ["F4010"]);
    assert_eq!(records[0]["phase"], "runtime");
    assert_eq!(records[0]["location"]["start"]["line"], 5);
}

#[test]
fn project_warnings_are_warning_records() {
    let cwd = create_temp_dir("json-warnings");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/util.fpas\", \"src/*.fpas\"]\n",
    );
    write_text(&cwd.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    write_text(&cwd.join("src/util.fpas"), "unit App.Util;\n");

    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &args(&[
            "check",
            "--diagnostics",
            "json",
            &project_file.to_string_lossy(),
        ]),
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr}");
    let records = records(&stderr);
    assert_eq!(codes(&records), ["F5035"]);
    assert_eq!(records[0]["severity"], "warning");
}

#[test]
fn test_command_emits_only_records_and_keeps_the_json_report_on_stdout() {
    let cwd = create_temp_dir("json-test");
    write_text(
        &cwd.join("fail_test.fpas"),
        "program F;\nuses Std.Test;\nbegin\n  AssertTrue(false)\nend.\n",
    );
    write_text(
        &cwd.join("pass_test.fpas"),
        "program P;\nuses Std.Test;\nbegin\n  AssertTrue(true)\nend.\n",
    );

    for jobs in ["1", "2"] {
        let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(
            &args(&[
                "test",
                "--diagnostics",
                "json",
                "--report",
                "json",
                "--jobs",
                jobs,
                &cwd.to_string_lossy(),
            ]),
            &cwd,
        );

        assert_eq!(exit_code, 1, "jobs {jobs}: {stderr}");
        let records = records(&stderr);
        assert_eq!(codes(&records), ["F4023"], "jobs {jobs}");
        assert!(
            records[0]["source"]
                .as_str()
                .expect("source")
                .ends_with("fail_test.fpas")
        );
        let report: Value = serde_json::from_str(&stdout).expect("JSON test report");
        assert_eq!(report["summary"]["failed"], 1);
        // Runtime locations are points: the start is known, the end is not.
        assert_eq!(records[0]["location"]["start"]["line"], 4);
        assert_eq!(records[0]["location"]["start"]["column"], 3);
        assert_eq!(records[0]["location"]["end"], Value::Null);
    }
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn argument_errors_use_the_requested_format() {
    let cwd = create_temp_dir("json-arguments");

    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &args(&["check", "--diagnostics", "json", "--bogus"]),
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    let records = records(&stderr);
    assert_eq!(codes(&records), ["F5037"]);
    assert!(
        records[0]["message"]
            .as_str()
            .expect("message")
            .contains("--bogus")
    );
}

#[test]
fn text_and_json_report_the_same_code_and_message() {
    let cwd = create_temp_dir("json-parity");
    let main = cwd.join("main.fpas");
    write_text(&main, "program Broken;\nbegin\n  MissingCall()\nend.\n");
    let path = main.to_string_lossy().into_owned();

    let (_, _, text) = support::run_cli_args_and_capture_output(&args(&["check", &path]), &cwd);
    let (_, _, json) = support::run_cli_args_and_capture_output(
        &args(&["check", "--diagnostics", "json", &path]),
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let record = &records(&json)[0];
    let message = record["message"].as_str().expect("message");
    assert!(text.contains(&format!(
        "error[{}]: {message}",
        record["code"].as_str().expect("code")
    )));
}
