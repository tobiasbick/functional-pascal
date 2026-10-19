//! Real-process coverage for the one-shot debugger's public result boundary.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "Integration fixtures and JSON shape assertions must fail immediately on setup or protocol errors."
)]

#[path = "batch_debug/inspection.rs"]
mod inspection;
#[path = "batch_debug/limits.rs"]
mod limits;
#[path = "batch_debug/requests.rs"]
mod requests;
#[path = "batch_debug/scheduling.rs"]
mod scheduling;
#[path = "batch_debug/trace.rs"]
mod trace;
#[path = "batch_debug/trace_callback_names.rs"]
mod trace_callback_names;
#[path = "batch_debug/trace_captures.rs"]
mod trace_captures;
#[path = "batch_debug/trace_fields.rs"]
mod trace_fields;
#[path = "batch_debug/trace_indices.rs"]
mod trace_indices;
#[path = "batch_debug/trace_limits.rs"]
mod trace_limits;
#[path = "batch_debug/trace_locals.rs"]
mod trace_locals;
#[path = "batch_debug/trace_projections.rs"]
mod trace_projections;

use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "fpas-batch-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let fixture = Self(root);
        fixture.write("main.fpas", source);
        fixture
    }
    fn write(&self, name: &str, contents: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_fpas"));
        command.current_dir(&self.0);
        command
    }
    fn run(&self, args: &[&str], exit: i32) -> Value {
        let child = self
            .command()
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let prefix = format!("fpas-debug-batch-{}-", child.id());
        let output = child.wait_with_output().unwrap();
        assert!(
            !fs::read_dir(std::env::temp_dir())
                .unwrap()
                .flatten()
                .any(|entry| entry.file_name().to_string_lossy().starts_with(&prefix)),
            "batch scratch was not removed"
        );
        report(output, exit)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn report(output: Output, exit: i32) -> Value {
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.status.code(),
        Some(exit),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stdout.ends_with(b"\n"));
    assert!(output.stdout.len() <= 8 * 1024 * 1024);
    let report: Value = serde_json::from_slice(&output.stdout).expect("one complete JSON document");
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["exit_code"], exit);
    for key in [
        "outcome",
        "stop",
        "diagnostics",
        "sources",
        "globals",
        "tasks",
        "expressions",
        "output",
        "capture",
        "trace",
        "limits",
        "schedule",
    ] {
        assert!(report.get(key).is_some(), "missing {key}");
    }
    report
}
fn text(report: &Value) -> String {
    report["output"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["text"].as_str().unwrap())
        .collect()
}
const HELLO: &str = "program Hello; uses Std.Console as Console; var Total: integer := 1; begin Total := 42; Console.WriteLn('hello'); end.";

#[test]
fn completion_keeps_globals_and_captures_output_once() {
    let f = Fixture::new(HELLO);
    let r = f.run(&["debug", "main.fpas", "--print", "Total"], 0);
    assert_eq!(r["outcome"], "completed");
    assert_eq!(r["expressions"][0]["value"]["scalar"], "42");
    assert_eq!(text(&r), "hello\n");
    assert!(r["tasks"][0]["frames"].as_array().unwrap().is_empty());
}
#[test]
fn large_completed_output_is_fully_acknowledged() {
    let f = Fixture::new(
        "program Large; uses Std.Console as Console; begin Console.WriteText('x'.RepeatStr(400000)); end.",
    );
    let r = f.run(&["debug", "main.fpas"], 0);
    assert_eq!(text(&r).len(), 400000);
}
#[cfg(unix)]
#[test]
fn native_output_preserves_split_utf8_and_stderr_is_inside_json() {
    let f = Fixture::new(
        "program Native; uses Std.Proc as Proc; begin discard Proc.Run('sh', ['-c', 'printf \"\\303\"; sleep 0.1; printf \"\\251\"; printf problem >&2']).Unwrap(); end.",
    );
    let r = f.run(&["debug", "main.fpas"], 0);
    let records = r["output"].as_array().unwrap();
    let stdout: String = records
        .iter()
        .filter(|v| v["stream"] == "stdout")
        .map(|v| v["text"].as_str().unwrap())
        .collect();
    let stderr: String = records
        .iter()
        .filter(|v| v["stream"] == "stderr")
        .map(|v| v["text"].as_str().unwrap())
        .collect();
    assert_eq!(stdout, "é");
    assert_eq!(stderr, "problem");
}

#[cfg(unix)]
#[test]
fn native_and_hosted_stdout_share_program_write_order() {
    let f = Fixture::new(
        "program Ordered; uses Std.Console as Console, Std.Proc as Proc; begin Console.WriteText('A'); discard Proc.Run('sh', ['-c', 'printf B']).Unwrap(); Console.WriteText('C'); end.",
    );
    let r = f.run(&["debug", "main.fpas"], 0);
    assert_eq!(text(&r), "ABC");
}
