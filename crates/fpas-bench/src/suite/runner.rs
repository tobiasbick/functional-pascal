//! Run benchmark processes with bounded execution and captured diagnostics.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use super::{BenchRun, BenchSpec};
use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const OUTPUT_DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

struct CapturedOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

struct PipeReader {
    receiver: Receiver<io::Result<Vec<u8>>>,
    thread: JoinHandle<()>,
}

/// Run one suite entry and parse its elapsed time.
pub fn run_bench(repo_root: &Path, fpas: &Path, spec: &BenchSpec) -> Result<BenchRun, String> {
    let mut command = super::command::command(repo_root, fpas, spec)?;
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = spawn_benchmark(&mut command)
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

fn spawn_benchmark(command: &mut Command) -> io::Result<Child> {
    configure_process_tree(command);
    command.spawn()
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
                stdout: finish_reader(stdout_reader, "stdout")?,
                stderr: finish_reader(stderr_reader, "stderr")?,
            });
        }
        if started.elapsed() >= timeout {
            let (termination_error, wait_error) = terminate_process_tree(&mut child);
            let stdout = finish_reader(stdout_reader, "stdout")?;
            let stderr = finish_reader(stderr_reader, "stderr")?;
            let mut message = format!(
                "benchmark `{benchmark_id}` exceeded its {} ms timeout and was terminated\n--- stdout ---\n{}\n--- stderr ---\n{}",
                timeout.as_millis(),
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            );
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

fn read_pipe<R>(mut pipe: R) -> PipeReader
where
    R: Read + Send + 'static,
{
    let (sender, receiver) = mpsc::sync_channel(1);
    let thread = thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = pipe.read_to_end(&mut bytes).map(|_| bytes);
        let _ = sender.send(result);
    });
    PipeReader { receiver, thread }
}

fn finish_reader(reader: PipeReader, stream: &str) -> Result<Vec<u8>, String> {
    let result = match reader.receiver.recv_timeout(OUTPUT_DRAIN_TIMEOUT) {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => {
            return Err(format!(
                "benchmark {stream} remained open after process termination"
            ));
        }
        Err(RecvTimeoutError::Disconnected) => {
            return Err(format!("benchmark {stream} reader panicked"));
        }
    };
    reader
        .thread
        .join()
        .map_err(|_| format!("benchmark {stream} reader panicked"))?;
    result.map_err(|error| format!("failed to read benchmark {stream}: {error}"))
}

fn configure_process_tree(command: &mut Command) {
    #[cfg(unix)]
    unix::configure(command);
    #[cfg(windows)]
    windows::configure(command);
}

fn terminate_process_tree(child: &mut Child) -> (Option<String>, Option<io::Error>) {
    #[cfg(unix)]
    return unix::terminate(child);
    #[cfg(windows)]
    return windows::terminate(child);
    #[cfg(not(any(unix, windows)))]
    {
        let termination_error = child.kill().err().map(|error| error.to_string());
        let wait_error = child.wait().err();
        (termination_error, wait_error)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_elapsed_ms, spawn_benchmark, wait_for_output};
    use std::error::Error;
    use std::fs;
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
    fn timeout_tree_parent_fixture() -> Result<(), Box<dyn Error>> {
        let Some(ready_path) = std::env::var_os("FPAS_BENCH_TIMEOUT_TREE_READY") else {
            return Ok(());
        };
        let executable = std::env::current_exe()?;
        Command::new(executable)
            .args([
                "--exact",
                "suite::runner::tests::timeout_tree_grandchild_fixture",
                "--nocapture",
            ])
            .env("FPAS_BENCH_TIMEOUT_TREE_GRANDCHILD", "1")
            .env("FPAS_BENCH_TIMEOUT_TREE_READY", ready_path)
            .spawn()?;
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    #[test]
    fn timeout_tree_grandchild_fixture() -> Result<(), Box<dyn Error>> {
        if std::env::var_os("FPAS_BENCH_TIMEOUT_TREE_GRANDCHILD").is_none() {
            return Ok(());
        }
        println!("grandchild started");
        let ready_path = std::env::var_os("FPAS_BENCH_TIMEOUT_TREE_READY")
            .ok_or_else(|| io::Error::other("missing timeout tree ready path"))?;
        fs::write(ready_path, [])?;
        std::thread::sleep(Duration::from_secs(2));
        Ok(())
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

    #[test]
    fn timeout_terminates_descendants_without_waiting_for_inherited_pipes()
    -> Result<(), Box<dyn Error>> {
        let ready_path = std::env::temp_dir().join(format!(
            "fpas-bench-timeout-tree-ready-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&ready_path);
        let executable = std::env::current_exe()?;
        let mut command = Command::new(executable);
        command
            .args([
                "--exact",
                "suite::runner::tests::timeout_tree_parent_fixture",
                "--nocapture",
            ])
            .env("FPAS_BENCH_TIMEOUT_TREE_READY", &ready_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let child = spawn_benchmark(&mut command)?;

        let ready_deadline = Instant::now() + Duration::from_secs(5);
        while !ready_path.is_file() && Instant::now() < ready_deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        if !ready_path.is_file() {
            return Err(io::Error::other("grandchild fixture did not start").into());
        }

        let started = Instant::now();
        let result = wait_for_output(child, Duration::from_millis(100), "process-tree");
        let elapsed = started.elapsed();
        let _ = fs::remove_file(&ready_path);
        let error = match result {
            Ok(_) => return Err(io::Error::other("fixture should time out").into()),
            Err(error) => error,
        };

        assert!(error.contains("grandchild started"), "{error}");
        assert!(
            elapsed < Duration::from_secs(1),
            "timeout waited {elapsed:?} for an inherited pipe"
        );
        Ok(())
    }
}
