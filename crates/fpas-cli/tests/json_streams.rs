//! Real-process JSON diagnostic streams for CLI commands and bundled native applications.
//!
//! Documentation: `docs/pascal/program-structure/cli.md` (machine-readable diagnostics).

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "process fixtures assert directly on spawned command output"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

fn temp_dir() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let path = std::env::temp_dir().join(format!(
        "fpas-json-streams-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).expect("temporary directory must be created");
    path
}

fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory must be created");
    }
    fs::write(path, text).expect("fixture must be written");
}

/// A program that prints to stdout, runs a child writing `child err` to stderr, then panics.
fn program_source() -> String {
    let (command, script) = if cfg!(windows) {
        ("cmd", "['/C', 'echo child err>&2']")
    } else {
        ("sh", "['-c', 'echo child err >&2']")
    };
    format!(
        "program Main;\nuses Std.Console, Std.Proc, Std.Args;\nbegin\n  WriteLn('out');\n  if ParamCount() > 0 then WriteLn(ParamStr(0));\n  case Run('{command}', {script}) of\n    Ok(Code): WriteLn(Code);\n    Error(Message): WriteLn(Message);\n  end;\n  panic('boom');\nend.\n"
    )
}

fn stderr_records(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .map(|line| {
            serde_json::from_str(line)
                .unwrap_or_else(|error| panic!("stderr line is not JSON ({error}): {line}"))
        })
        .collect()
}

fn assert_program_output_then_runtime_record(output: &Output, stdout: &str) {
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), stdout);
    let records = stderr_records(output);
    assert_eq!(records.len(), 2, "{records:?}");
    assert_eq!(records[0]["kind"], "program-output");
    assert_eq!(records[0]["stream"], "stderr");
    assert_eq!(records[0]["text"], "child err");
    assert_eq!(records[1]["kind"], "diagnostic");
    assert_eq!(records[1]["code"], "FP5010");
    assert_eq!(records[1]["phase"], "runtime");
}

#[test]
fn run_wraps_child_stderr_as_program_output_in_json_mode() {
    let root = temp_dir();
    let main = root.join("main.fpas");
    write(&main, &program_source());

    let json = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(&root)
        .args(["run", "--diagnostics", "json", "main.fpas"])
        .output()
        .expect("fpas run must start");
    let text = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(&root)
        .args(["run", "main.fpas"])
        .output()
        .expect("fpas run must start");
    fs::remove_dir_all(&root).expect("temporary directory must be removed");

    assert_program_output_then_runtime_record(&json, "out\n0\n");
    // Text mode keeps the inherited child stderr unchanged.
    let text_stderr = String::from_utf8_lossy(&text.stderr);
    assert_eq!(text.status.code(), Some(2));
    assert!(text_stderr.starts_with("child err"), "{text_stderr}");
    assert!(
        text_stderr.contains("error[FP5010]: panic: boom"),
        "{text_stderr}"
    );
}

#[test]
fn native_runner_selects_json_through_the_environment() {
    let root = temp_dir();
    write(
        &root.join("app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    write(&root.join("src/main.fpas"), &program_source());
    let build = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(&root)
        .args(["build", "--executable", "--name", "app", "app.fpasprj"])
        .output()
        .expect("fpas build must start");
    assert!(
        build.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let executable = root.join(if cfg!(windows) { "app.exe" } else { "app" });

    let json = Command::new(&executable)
        .current_dir(&root)
        .env("FPAS_DIAGNOSTICS", "json")
        .arg("--diagnostics")
        .output()
        .expect("bundled application must start");
    let text = Command::new(&executable)
        .current_dir(&root)
        .env_remove("FPAS_DIAGNOSTICS")
        .output()
        .expect("bundled application must start");
    fs::remove_dir_all(&root).expect("temporary directory must be removed");

    // Application arguments such as `--diagnostics` are passed through untouched.
    assert_program_output_then_runtime_record(&json, "out\n--diagnostics\n0\n");
    assert!(
        String::from_utf8_lossy(&text.stderr).contains("error[FP5010]: panic: boom"),
        "{text:?}"
    );
}

fn command_output(root: &Path, command: &str, input: &str, json: bool) -> Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_fpas"));
    process.current_dir(root).arg(command);
    if json {
        process.args(["--diagnostics", "json"]);
    }
    process.arg(input).output().expect("CLI process must start")
}

#[test]
fn all_four_commands_preserve_source_failures_in_text_and_json() {
    let root = temp_dir();
    write(
        &root.join("app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main_test.fpas\"\n\n[sources]\ninclude = [\"main_test.fpas\"]\n",
    );
    for (source, expected) in [
        ("program P; begin @ end.", "FP1001"),
        ("program P begin end.", "FP2001"),
        ("program P; begin var N: integer := 'hello'; end.", "FP3006"),
        (
            "program P; procedure Print(A, B: integer); begin end; begin end.",
            "FP2014",
        ),
        (
            "program P; uses Std.Console; begin WriteLn('ä😀'); MissingCall(); end.",
            "FP3003",
        ),
    ] {
        write(&root.join("main_test.fpas"), source);
        for command in ["check", "build", "run", "test"] {
            let input = if command == "build" {
                "app.fpasprj"
            } else {
                "main_test.fpas"
            };
            let text = command_output(&root, command, input, false);
            let json = command_output(&root, command, input, true);
            assert_eq!(
                json.status.code(),
                Some(if command == "test" { 2 } else { 1 }),
                "{command}: {json:?}"
            );
            assert_eq!(text.status.code(), json.status.code());
            assert_eq!(text.stdout, json.stdout, "{command} stdout");
            let records = stderr_records(&json);
            let record = records
                .iter()
                .find(|record| record["code"] == expected)
                .unwrap_or_else(|| panic!("{command}: {records:?}"));
            assert!(String::from_utf8_lossy(&text.stderr).contains(&format!(
                "error[{expected}]: {}",
                record["message"].as_str().expect("message")
            )));
            assert!(
                record["source"]
                    .as_str()
                    .expect("path")
                    .ends_with("main_test.fpas")
            );
            assert_eq!(record["location"]["start"]["line"], 1);
            assert!(record["location"]["end"].is_object());
            if expected == "FP3006" {
                assert_eq!(record["expected"], "integer");
                assert_eq!(record["found"], "string");
            }
            if expected == "FP2014" {
                assert_eq!(record["found"], ",");
                assert!(
                    record["hint"]
                        .as_str()
                        .expect("hint")
                        .contains("A: integer; B: integer")
                );
            }
            if expected == "FP3003" {
                assert_eq!(
                    record["location"]["start"]["column"],
                    source[..source.find("MissingCall").expect("call")]
                        .chars()
                        .count()
                        + 1
                );
            }
        }
    }
    fs::remove_dir_all(root).expect("remove fixtures");
}

#[test]
fn real_process_records_keep_imported_paths_multiple_errors_and_absent_positions() {
    let root = temp_dir();
    write(
        &root.join("app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/*.fpas\"]\n",
    );
    write(
        &root.join("src/main.fpas"),
        "program P; uses App.Bad; begin end.",
    );
    write(
        &root.join("src/bad.fpas"),
        "unit App.Bad public procedure P(); begin @ end;",
    );
    for command in ["check", "build", "run"] {
        let output = command_output(&root, command, "app.fpasprj", true);
        assert_eq!(output.status.code(), Some(1));
        let records = stderr_records(&output);
        assert!(
            records.iter().any(|record| record["code"] == "FP1001"),
            "{records:?}"
        );
        assert!(
            records.iter().any(|record| record["code"] == "FP2001"),
            "{records:?}"
        );
        for record in records {
            assert!(
                record["source"]
                    .as_str()
                    .expect("imported path")
                    .ends_with("bad.fpas"),
                "{record}"
            );
        }
    }
    write(
        &root.join("app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"unknown\"\n",
    );
    let output = command_output(&root, "build", "app.fpasprj", true);
    let records = stderr_records(&output);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["code"], "FP4105");
    assert_eq!(records[0]["location"], Value::Null);
    assert!(
        records[0]["source"]
            .as_str()
            .expect("manifest path")
            .ends_with("app.fpasprj")
    );
    fs::remove_dir_all(root).expect("remove fixtures");
}

#[test]
fn runner_startup_failure_is_a_positionless_json_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_fpas-runner"))
        .env("FPAS_DIAGNOSTICS", "json")
        .output()
        .expect("runner starts");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let records = stderr_records(&output);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["code"], "FP4125");
    assert_eq!(records[0]["location"], Value::Null);
}

#[test]
fn malformed_record_initializer_finishes_with_bounded_diagnostics() {
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let root = temp_dir();
    write(
        &root.join("bad.fpas"),
        "program P; begin var Value: integer := record then end end.",
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(&root)
        .args(["check", "--diagnostics", "json", "bad.fpas"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("CLI starts");
    let deadline = Instant::now() + Duration::from_secs(3);
    while child.try_wait().expect("poll CLI").is_none() {
        if Instant::now() >= deadline {
            child.kill().expect("stop hung parser");
            child.wait().expect("reap hung parser");
            fs::remove_dir_all(root).expect("remove fixtures");
            panic!("malformed record initializer did not produce bounded diagnostics in 3 seconds");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().expect("CLI output");
    fs::remove_dir_all(root).expect("remove fixtures");
    assert_eq!(output.status.code(), Some(1));
    let records = stderr_records(&output);
    assert!(records.len() < 16, "{records:?}");
    assert!(
        records.iter().any(|record| record["code"] == "FP2005"),
        "{records:?}"
    );
}

#[path = "json_streams/test_workers.rs"]
mod test_workers;
