//! Raw-mode selection and parent-owned query deadlines, including parser failures.

use super::protocol::{Outcome, Report};
use super::report;
use crate::oneshot::{Files, Request};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub(crate) fn selected(args: &[String]) -> bool {
    matches!(args.first().map(String::as_str), Some("query" | "refactor"))
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
    let timeout = report.limits.timeout_ms;
    if started.elapsed() >= Duration::from_millis(timeout) {
        let failed = report.outcome == Outcome::Error && report.exit_code == 3;
        report::incomplete(&mut report, "execution_timeout");
        if failed {
            report.outcome = Outcome::Error;
        }
        bytes = report::finish(&mut report);
    } else if let Ok(crate::ResolvedCli::Refactor(config)) = crate::resolve_cli_config(args, cwd)
        && let Some(deadline) = started.checked_add(Duration::from_millis(timeout))
    {
        // After this coordinator begins replacement, the preparation timeout cannot kill
        // publication/recovery or relabel a successfully delivered edit as a timeout.
        super::refactor::coordinate(&config, &mut report, deadline);
        bytes = report::finish(&mut report);
    }
    let exit = report.exit_code;
    if let Err(error) = stdout.write_all(&bytes) {
        let _ = if args.first().is_some_and(|a| a == "refactor") {
            writeln!(
                stderr,
                "Cannot deliver refactoring report: {error}. Inspect saved files before retrying."
            )
        } else {
            writeln!(stderr, "Cannot write query output: {error}")
        };
        return 1;
    }
    exit
}

fn parent(args: &[String], cwd: &Path, started: Instant) -> Report {
    let config = match crate::resolve_cli_config(args, cwd) {
        Ok(config @ (crate::ResolvedCli::Query(_) | crate::ResolvedCli::Refactor(_))) => config,
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
    let Some(deadline) = started.checked_add(Duration::from_secs(timeout(&config))) else {
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
        crate::process_tree::configure(&mut command)?;
        command.spawn()
    };
    let mut child = match spawn() {
        Ok(child) => child,
        Err(error) => return failure("worker_start", &error.to_string(), 3),
    };
    let (result, ended) = crate::oneshot::wait(&mut child, &files, deadline, || false);
    let mut report = match result
        .and_then(|value| serde_json::from_value::<Report>(value).map_err(|e| e.to_string()))
    {
        Ok(report) if report.schema_version == 2 && [0, 1, 3].contains(&report.exit_code) => report,
        Ok(_) => report::failure(args, cwd, "worker_failure", "Invalid worker result.", 3),
        Err(error) => report::failure(args, cwd, "worker_failure", &error, 3),
    };
    effective_limits(&mut report, &config);
    if let Some(reason) = ended {
        let failed = report.outcome == Outcome::Error;
        report::incomplete(&mut report, reason);
        if failed {
            report.outcome = Outcome::Error;
        }
        report::diagnostic(
            &mut report,
            reason,
            "The invocation deadline expired; worker state is unavailable.",
        );
    }
    report
}

fn timeout(config: &crate::ResolvedCli) -> u64 {
    match config {
        crate::ResolvedCli::Query(c) => c.timeout,
        crate::ResolvedCli::Refactor(c) => c.timeout,
        _ => 30,
    }
}

fn effective_limits(report: &mut Report, config: &crate::ResolvedCli) {
    report.limits.timeout_ms = timeout(config).saturating_mul(1000);
    let crate::ResolvedCli::Query(config) = config else {
        report.limits.result_count = None;
        return;
    };
    report.limits.timeout_ms = config.timeout.saturating_mul(1000);
    report.limits.result_count = matches!(
        config.kind,
        crate::cli_input::QueryKind::Refs
            | crate::cli_input::QueryKind::Find
            | crate::cli_input::QueryKind::Callers
            | crate::cli_input::QueryKind::Callees
    )
    .then_some(config.limit);
}
