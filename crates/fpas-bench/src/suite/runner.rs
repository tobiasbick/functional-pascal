//! Run benchmark processes with bounded execution and captured diagnostics.

use super::{BenchEngine, BenchRun, BenchSpec};
use fpas_bytecode::VerifiedExecutable;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

struct CapturedOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

/// Run one suite entry and parse its elapsed time.
pub fn run_bench(repo_root: &Path, fpas: &Path, spec: &BenchSpec) -> Result<BenchRun, String> {
    if spec.engine == BenchEngine::Register {
        return run_register_bench(repo_root, spec);
    }

    let program = repo_root.join(&spec.path);
    if !program.is_file() {
        return Err(format!("benchmark source missing: {}", program.display()));
    }

    let mut command = Command::new(fpas);
    command
        .arg("run")
        .arg(&program)
        .arg("--")
        .args(&spec.args)
        .current_dir(repo_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command
        .spawn()
        .map_err(|error| format!("failed to spawn {}: {error}", fpas.display()))?;
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

fn run_register_bench(repo_root: &Path, spec: &BenchSpec) -> Result<BenchRun, String> {
    let program_path = repo_root.join(&spec.path);
    let source = fs::read_to_string(&program_path)
        .map_err(|error| format!("failed to read {}: {error}", program_path.display()))?;
    let (program, diagnostics) = fpas_parser::parse(&source);
    if let Some(error) = diagnostics.first() {
        let diagnostic = error.as_diagnostic();
        return Err(format!(
            "register benchmark `{}` did not parse: {}: {}",
            spec.id, diagnostic.code, diagnostic.message
        ));
    }
    let executable = fpas_compiler::compile_register_subset(&program)
        .map_err(|errors| format_diagnostics("compile", &spec.id, &errors))?;
    execute_register_bench(spec, executable)
}

fn execute_register_bench(
    spec: &BenchSpec,
    executable: VerifiedExecutable,
) -> Result<BenchRun, String> {
    let started = Instant::now();
    let execution = fpas_vm::RegisterVm::new(executable)
        .run()
        .map_err(|error| {
            format!(
                "register benchmark `{}` failed: {}: {}",
                spec.id, error.code, error.message
            )
        })?;
    let elapsed_ms = u64::try_from(started.elapsed().as_millis())
        .unwrap_or(u64::MAX)
        .max(1);
    let operations_per_second = execution
        .instruction_count
        .saturating_mul(1_000)
        .checked_div(elapsed_ms)
        .unwrap_or(0);
    let throughput = format!("throughput: {operations_per_second} instructions/s");
    let raw_stdout = format!(
        "elapsed: {elapsed_ms} ms\n{throughput}\ninstructions: {}\n",
        execution.instruction_count
    );
    Ok(BenchRun {
        id: spec.id.clone(),
        elapsed_ms,
        throughput: Some(throughput),
        raw_stdout,
    })
}

fn format_diagnostics(
    stage: &str,
    benchmark_id: &str,
    errors: &[fpas_compiler::CompileError],
) -> String {
    let details = errors
        .iter()
        .map(|error| format!("{}: {}", error.code, error.message))
        .collect::<Vec<_>>()
        .join("; ");
    format!("register benchmark `{benchmark_id}` failed to {stage}: {details}")
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
    mut child: Child,
    timeout: Duration,
    benchmark_id: &str,
) -> Result<CapturedOutput, String> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "benchmark stdout was not captured".to_owned())?;
    let stderr = child
        .stderr
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
            return Ok(CapturedOutput {
                status,
                stdout: join_reader(stdout_reader, "stdout")?,
                stderr: join_reader(stderr_reader, "stderr")?,
            });
        }
        if started.elapsed() >= timeout {
            let kill_error = child.kill().err();
            let wait_error = child.wait().err();
            let stdout = join_reader(stdout_reader, "stdout")?;
            let stderr = join_reader(stderr_reader, "stderr")?;
            let mut message = format!(
                "benchmark `{benchmark_id}` exceeded its {} ms timeout and was terminated\n--- stdout ---\n{}\n--- stderr ---\n{}",
                timeout.as_millis(),
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            );
            if let Some(error) = kill_error {
                message.push_str(&format!("\nfailed to terminate process: {error}"));
            }
            if let Some(error) = wait_error {
                message.push_str(&format!("\nfailed to reap process: {error}"));
            }
            return Err(message);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn read_pipe<R>(mut pipe: R) -> JoinHandle<io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

fn join_reader(reader: JoinHandle<io::Result<Vec<u8>>>, stream: &str) -> Result<Vec<u8>, String> {
    reader
        .join()
        .map_err(|_| format!("benchmark {stream} reader panicked"))?
        .map_err(|error| format!("failed to read benchmark {stream}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{parse_elapsed_ms, wait_for_output};
    use std::error::Error;
    use std::io;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    #[test]
    fn parse_elapsed_ms_reads_standard_line() {
        let stdout = "iterations: 1\nelapsed: 142 ms\nthroughput: 7 ops/s\n";
        assert_eq!(parse_elapsed_ms(stdout), Some(142));
    }

    #[test]
    fn parse_elapsed_ms_ignores_noise() {
        assert_eq!(parse_elapsed_ms("no timing here\n"), None);
    }

    #[test]
    fn timeout_child_fixture() {
        if std::env::var_os("FPAS_BENCH_TIMEOUT_FIXTURE").is_none() {
            return;
        }
        println!("child started");
        eprintln!("child waiting");
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    #[test]
    fn timeout_terminates_process_and_retains_diagnostics() -> Result<(), Box<dyn Error>> {
        let executable = std::env::current_exe()?;
        let child = Command::new(executable)
            .args([
                "--exact",
                "suite::runner::tests::timeout_child_fixture",
                "--nocapture",
            ])
            .env("FPAS_BENCH_TIMEOUT_FIXTURE", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let started = Instant::now();
        let result = wait_for_output(child, Duration::from_millis(100), "hung");
        let elapsed = started.elapsed();
        let error = match result {
            Ok(_) => return Err(io::Error::other("fixture should time out").into()),
            Err(error) => error,
        };

        assert_eq!(
            (
                error.contains("exceeded its 100 ms timeout"),
                error.contains("child started"),
                error.contains("child waiting"),
                elapsed < Duration::from_secs(5)
            ),
            (true, true, true, true)
        );
        Ok(())
    }
}
