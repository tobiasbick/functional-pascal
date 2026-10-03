//! Real-process JSON diagnostic streams for `fpas run` and bundled native applications.
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

#[path = "json_streams/operators.rs"]
mod operators;

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
        "program Main;\nuses Std.Console as Console; uses Std.Proc as Proc;\nbegin\n  Console.WriteLn('out');\n  case Proc.Run('{command}', {script}) of\n    when Ok(Code): Console.WriteLn(Code);\n    when Error(Message): Console.WriteLn(Message);\n  end case;\n  panic('boom');\nend program;\n"
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

fn assert_program_output_then_runtime_record(output: &Output) {
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "out\n0\n");
    let records = stderr_records(output);
    assert_eq!(records.len(), 2, "{records:?}");
    assert_eq!(records[0]["kind"], "program-output");
    assert_eq!(records[0]["stream"], "stderr");
    assert_eq!(records[0]["text"], "child err");
    assert_eq!(records[1]["kind"], "diagnostic");
    assert_eq!(records[1]["code"], "F4010");
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

    assert_program_output_then_runtime_record(&json);
    // Text mode keeps the inherited child stderr unchanged.
    let text_stderr = String::from_utf8_lossy(&text.stderr);
    assert_eq!(text.status.code(), Some(2));
    assert!(text_stderr.starts_with("child err"), "{text_stderr}");
    assert!(
        text_stderr.contains("error[F4010]: panic: boom"),
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
    assert_program_output_then_runtime_record(&json);
    assert!(
        String::from_utf8_lossy(&text.stderr).contains("error[F4010]: panic: boom"),
        "{text:?}"
    );
}
