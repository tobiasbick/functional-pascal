//! Run benchmark processes with bounded execution and captured diagnostics.

use super::{BenchRun, BenchSpec};
#[cfg(windows)]
use process_wrap::std::JobObject;
#[cfg(unix)]
use process_wrap::std::ProcessGroup;
use process_wrap::std::{ChildWrapper, CommandWrap};
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

mod output;
mod termination;

use output::{finish_reader, read_pipe};
use termination::terminate_process_tree;

#[derive(Debug)]
struct CapturedOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

/// Run one suite entry and parse its elapsed time.
pub fn run_bench(repo_root: &Path, fpas: &Path, spec: &BenchSpec) -> Result<BenchRun, String> {
    let mut command = super::command::command(repo_root, fpas, spec)?;
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = spawn_benchmark(command)
        .map_err(|error| format!("failed to spawn benchmark `{}`: {error}", spec.id))?;
    let output = wait_for_output(child, Duration::from_millis(spec.timeout_ms), &spec.id)?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        return Err(format!(
            "benchmark `{}` failed ({})\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
            spec.id, output.status
        ));
    }

    let elapsed_ms = parse_elapsed_ms(&stdout).ok_or_else(|| {
        format!(
            "benchmark `{}` stdout missing `elapsed: N ms` line:\n{stdout}",
            spec.id
        )
    })?;
    let throughput = parse_throughput_line(&stdout);

    Ok(BenchRun {
        id: spec.id.clone(),
        elapsed_ms,
        throughput,
        raw_stdout: stdout,
    })
}

fn spawn_benchmark(command: Command) -> io::Result<Box<dyn ChildWrapper>> {
    let mut wrapped = CommandWrap::from(command);
    #[cfg(unix)]
    wrapped.wrap(ProcessGroup::leader());
    #[cfg(windows)]
    wrapped.wrap(JobObject);
    wrapped.spawn()
}

/// Run all filtered specs in order.
pub fn run_suite(
    repo_root: &Path,
    fpas: &Path,
    specs: &[BenchSpec],
) -> Result<Vec<BenchRun>, String> {
    let mut runs = Vec::with_capacity(specs.len());
    for spec in specs {
        eprintln!("running {}…", spec.id);
        runs.push(run_bench(repo_root, fpas, spec)?);
    }
    Ok(runs)
}

/// Parse `elapsed: <int> ms` from benchmark stdout.
pub fn parse_elapsed_ms(stdout: &str) -> Option<u64> {
    for line in stdout.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("elapsed:") else {
            continue;
        };
        let rest = rest.trim();
        let Some(number) = rest.strip_suffix("ms") else {
            continue;
        };
        if let Ok(value) = number.trim().parse::<u64>() {
            return Some(value);
        }
    }
    None
}

fn parse_throughput_line(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("throughput:"))
        .map(str::to_owned)
}

fn wait_for_output(
    mut child: Box<dyn ChildWrapper>,
    timeout: Duration,
    benchmark_id: &str,
) -> Result<CapturedOutput, String> {
    let stdout = child
        .stdout()
        .take()
        .ok_or_else(|| "benchmark stdout was not captured".to_owned())?;
    let stderr = child
        .stderr()
        .take()
        .ok_or_else(|| "benchmark stderr was not captured".to_owned())?;
    let stdout_reader = read_pipe(stdout);
    let stderr_reader = read_pipe(stderr);
    let started = Instant::now();

    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to poll benchmark `{benchmark_id}`: {error}"))?
        {
            let _ = child.start_kill();
            return Ok(CapturedOutput {
                status,
                stdout: finish_reader(stdout_reader, "stdout")?,
                stderr: finish_reader(stderr_reader, "stderr")?,
            });
        }
        if started.elapsed() >= timeout {
            let (termination_error, wait_error) = terminate_process_tree(child.as_mut());
            let stdout = finish_reader(stdout_reader, "stdout");
            let stderr = finish_reader(stderr_reader, "stderr");
            let mut message = format!(
                "benchmark `{benchmark_id}` exceeded its {} ms timeout; process cleanup was attempted\n--- stdout ---\n{}\n--- stderr ---\n{}",
                timeout.as_millis(),
                stdout
                    .as_ref()
                    .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                    .unwrap_or_default(),
                stderr
                    .as_ref()
                    .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                    .unwrap_or_default()
            );
            for error in [stdout.err(), stderr.err()].into_iter().flatten() {
                message.push_str(&format!("\n{error}"));
            }
            if let Some(error) = termination_error {
                message.push_str(&format!("\nfailed to terminate process tree: {error}"));
            }
            if let Some(error) = wait_error {
                message.push_str(&format!("\nfailed to reap process: {error}"));
            }
            return Err(message);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests;
