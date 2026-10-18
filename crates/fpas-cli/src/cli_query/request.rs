//! Raw-mode selection and parent-owned query deadlines, including parser failures.

use super::report;
use crate::oneshot::{Files, Request};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub(crate) fn selected(args: &[String]) -> bool {
    args.first().map(String::as_str) == Some("query")
        && args.get(1).map(String::as_str) != Some("rename")
        && !args
            .iter()
            .any(|s| ["--help", "-h", "--version", "-V"].contains(&s.as_str()))
}

pub(crate) fn cwd_failure(args: &[String], error: &std::io::Error) -> Vec<u8> {
    let mut report = report::failure(
        args,
        Path::new(""),
        "current_directory",
        &error.to_string(),
        1,
    );
    report["context"]["path_base"] = Value::Null;
    report::finish(&mut report)
}

pub(crate) fn run(
    args: &[String],
    cwd: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let started = Instant::now();
    let mut report = parent(args, cwd, started);
    let mut bytes = report::finish(&mut report);
    let timeout = report["limits"]["timeout_seconds"].as_u64().unwrap_or(30);
    if started.elapsed() >= Duration::from_secs(timeout) {
        let failed = report["outcome"] == "error" && report["exit_code"] == 3;
        report::incomplete(&mut report, "execution_timeout");
        if failed {
            report["outcome"] = json!("error");
        }
        bytes = report::finish(&mut report);
    }
    let exit = report["exit_code"].as_i64().unwrap_or(3) as i32;
    if let Err(error) = stdout.write_all(&bytes) {
        let _ = writeln!(stderr, "Cannot write query output: {error}");
        return 1;
    }
    exit
}

fn parent(args: &[String], cwd: &Path, started: Instant) -> Value {
    let config = match crate::resolve_cli_config(args, cwd) {
        Ok(crate::ResolvedCli::Query(config)) => config,
        Ok(_) => {
            return report::failure(args, cwd, "invalid_request", "Expected a source query.", 1);
        }
        Err(error) => return report::failure(args, cwd, "invalid_request", &error, 1),
    };
    let failure = |code, message: &str, exit| {
        let mut report = report::failure(args, cwd, code, message, exit);
        effective_limits(&mut report, &config);
        report
    };
    let Some(deadline) = started.checked_add(Duration::from_secs(config.timeout)) else {
        return failure("invalid_request", "Timeout is too large.", 1);
    };
    let files = match Files::create("query") {
        Ok(files) => files,
        Err(error) => return failure("worker_setup", &error.to_string(), 1),
    };
    let spawn = || -> std::io::Result<std::process::Child> {
        files.write_request(&Request {
            args: args.into(),
            cwd: cwd.into(),
        })?;
        let mut command = Command::new(crate::process_tree::executable()?);
        command
            .arg(super::worker::ARGUMENT)
            .arg(&files.root)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        crate::process_tree::configure(&mut command);
        command.spawn()
    };
    let mut child = match spawn() {
        Ok(child) => child,
        Err(error) => return failure("worker_start", &error.to_string(), 3),
    };
    let (result, ended) = crate::oneshot::wait(&mut child, &files, deadline, || false);
    let mut report = match result {
        Ok(report)
            if report["schema_version"] == 1
                && report["diagnostics"].is_array()
                && report["coverage"]["reasons"].is_array()
                && [0, 1, 3].iter().any(|e| report["exit_code"] == *e) =>
        {
            report
        }
        Ok(_) => report::failure(args, cwd, "worker_failure", "Invalid worker result.", 3),
        Err(error) => report::failure(args, cwd, "worker_failure", &error, 3),
    };
    effective_limits(&mut report, &config);
    if let Some(reason) = ended {
        let failed = report["outcome"] == "error";
        report::incomplete(&mut report, reason);
        if failed {
            report["outcome"] = json!("error");
        }
        report::diagnostic(
            &mut report,
            reason,
            "The invocation deadline expired; worker state is unavailable.",
        );
    }
    report
}

fn effective_limits(report: &mut Value, config: &crate::cli_input::QueryCliConfig) {
    report["limits"]["timeout_seconds"] = json!(config.timeout);
    report["limits"]["result_count"] = if config.kind == crate::cli_input::QueryKind::Refs {
        json!(config.limit)
    } else {
        Value::Null
    };
}
