//! Parent-owned one-shot debug execution and publication of a single JSON document.

mod capture;
mod utf8;
mod worker;
pub(crate) use worker::worker;

use crate::oneshot::{Files, Request};
use fpas_debug::batch::{diagnostic, empty_report, finish_report};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Selects default batch debugging before target discovery and full option parsing.
pub(crate) fn selected(args: &[String]) -> bool {
    if args.first().map(String::as_str) != Some("debug") {
        return false;
    }
    let args = args
        .iter()
        .take_while(|arg| *arg != "--")
        .collect::<Vec<_>>();
    if args
        .iter()
        .any(|arg| ["--help", "-h", "--version", "-V"].contains(&arg.as_str()))
    {
        return false;
    }
    !args
        .windows(2)
        .any(|pair| pair[0] == "--protocol" && ["dap", "jsonl"].contains(&pair[1].as_str()))
}

/// Executes one batch worker and publishes exactly one bounded result to stdout.
#[expect(
    clippy::unwrap_used,
    reason = "Batch worker and error results share the fixed array schema created by empty_report."
)]
pub(crate) fn run(
    args: &[String],
    cwd: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let started = Instant::now();
    let mut report = match run_parent(args, cwd, started) {
        Ok(report) => report,
        Err((exit, code, error)) => {
            let mut report = empty_report("error", exit);
            report["diagnostics"] = json!([diagnostic(code, "request", &error)]);
            report
        }
    };
    let mut bytes = finish_report(&mut report);
    let timeout = report["limits"]["timeout_seconds"].as_u64().unwrap_or(30);
    if report["outcome"] != "limit" && started.elapsed() >= Duration::from_secs(timeout) {
        report["outcome"] = json!("limit");
        report["exit_code"] = json!(3);
        report["capture"]["incomplete"] = json!(true);
        report["capture"]["reasons"]
            .as_array_mut()
            .unwrap()
            .push(json!("execution_timeout"));
        report["diagnostics"]
            .as_array_mut()
            .unwrap()
            .push(diagnostic(
                "execution_timeout",
                "publication",
                "The invocation deadline expired while preparing the result.",
            ));
        bytes = finish_report(&mut report);
    }
    let exit = report["exit_code"].as_i64().unwrap_or(3) as i32;
    if let Err(error) = stdout.write_all(&bytes) {
        let _ = writeln!(stderr, "Cannot write batch result: {error}");
        return 1;
    }
    exit
}

type Failure = (i32, &'static str, String);

#[expect(
    clippy::unwrap_used,
    reason = "The parent constructs its fixed-schema report using empty_report."
)]
fn run_parent(args: &[String], cwd: &Path, started: Instant) -> Result<Value, Failure> {
    let timeout = raw_number(args, "--timeout", 30)?;
    let instruction_limit = raw_number(args, "--instruction-limit", 100_000_000)?;
    let output_limit = raw_number(args, "--output-limit", 1_048_576)?;
    if output_limit > 1_048_576 {
        return Err((
            1,
            "FP4137",
            "Batch --output-limit cannot exceed 1048576 bytes.".into(),
        ));
    }
    let deadline = started.checked_add(Duration::from_secs(timeout)).ok_or((
        1,
        "FP4137",
        "Timeout is too large.".into(),
    ))?;
    let files =
        Files::create("debug-batch").map_err(|error| (1, "worker_setup", error.to_string()))?;
    files
        .write_request(&Request {
            args: args.into(),
            cwd: cwd.into(),
        })
        .map_err(|error| (1, "worker_setup", error.to_string()))?;
    let executable = crate::process_tree::executable()
        .map_err(|error| (3, "worker_start", error.to_string()))?;
    let mut command = Command::new(executable);
    command
        .arg(worker::ARGUMENT)
        .arg(&files.root)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    crate::process_tree::configure(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| (3, "worker_start", error.to_string()))?;
    let captured = Arc::new(Mutex::new(capture::Output::new(output_limit as usize)));
    let stdout = capture::drain(
        child.stdout.take().unwrap(),
        "stdout",
        Arc::clone(&captured),
    );
    let stderr = capture::drain(
        child.stderr.take().unwrap(),
        "stderr",
        Arc::clone(&captured),
    );
    let (result, mut ended) = crate::oneshot::wait(&mut child, &files, deadline, || {
        captured.lock().unwrap_or_else(|p| p.into_inner()).overflow
    });
    let result = result.and_then(|report| {
        if report["schema_version"] != 1
            || !report["capture"]["reasons"].is_array()
            || !report["diagnostics"].is_array()
            || !report["exit_code"].as_u64().is_some_and(|exit| exit <= 3)
        {
            return Err("Worker published an invalid result envelope.".into());
        }
        Ok(report)
    });
    let _ = stdout.join();
    let _ = stderr.join();
    let mut output = captured.lock().unwrap_or_else(|p| p.into_inner());
    if output.overflow {
        ended = Some("output_limit");
    }
    let mut report = match result {
        Ok(report) => report,
        Err(error) => {
            let mut report = empty_report("error", 3);
            report["diagnostics"] = json!([diagnostic("worker_failure", "worker", &error)]);
            report["capture"]["incomplete"] = json!(true);
            report["capture"]["reasons"] = json!(["state_unavailable"]);
            report
        }
    };
    report["output"] = json!(std::mem::take(&mut output.records));
    report["limits"]["timeout_seconds"] = json!(timeout);
    report["limits"]["instruction_limit"] = json!(instruction_limit);
    report["limits"]["output_bytes"] = json!(output_limit);
    if let Some(code) = ended {
        report["outcome"] = json!("limit");
        report["exit_code"] = json!(3);
        report["capture"]["incomplete"] = json!(true);
        report["capture"]["reasons"]
            .as_array_mut()
            .unwrap()
            .push(json!(code));
        report["diagnostics"]
            .as_array_mut()
            .unwrap()
            .push(diagnostic(
                code,
                "execution",
                "The batch execution bound ended this invocation.",
            ));
    }
    Ok(report)
}

fn raw_number(args: &[String], option: &str, default: u64) -> Result<u64, Failure> {
    let args = args
        .iter()
        .take_while(|arg| *arg != "--")
        .collect::<Vec<_>>();
    let mut value = None;
    for (index, arg) in args.iter().enumerate() {
        if arg.as_str() != option {
            continue;
        }
        let parsed = args
            .get(index + 1)
            .and_then(|text| text.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .ok_or_else(|| {
                (
                    1,
                    "FP4137",
                    format!("{option} requires a positive integer."),
                )
            })?;
        if value.replace(parsed).is_some() {
            return Err((1, "FP4137", format!("Duplicate {option}.")));
        }
    }
    Ok(value.unwrap_or(default))
}
