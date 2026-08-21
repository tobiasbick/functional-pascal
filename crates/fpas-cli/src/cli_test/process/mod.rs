//! Isolated process lifecycle for wall-clock-limited test VM runs.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../../docs/pascal/std/testing/test.md)

mod output;
#[cfg(test)]
mod tests;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use output::CappedBuffer;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use fpas_program::{Digest, ProgramIdentity, ProgramImage};
use serde::{Deserialize, Serialize};

use super::report::TestOutcome;
use super::run::program::{PreparedProgram, RunOutput, run_prepared_program};

const WORKER_ARGUMENT: &str = "__fpas-test-process";
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_CAPTURED_OUTPUT: usize = 8 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct WorkerRequest {
    test_path: PathBuf,
    source_paths: Option<Vec<PathBuf>>,
    script_override: Option<PathBuf>,
    manifest_override: Option<ManifestOverride>,
    display: String,
    output: RunOutput,
}

#[derive(Serialize, Deserialize)]
struct ManifestOverride {
    script: Option<PathBuf>,
    headless_graph: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct WorkerResponse {
    outcome: TestOutcome,
}

struct WorkerFiles {
    root: PathBuf,
}

impl WorkerFiles {
    fn create() -> Result<Self, String> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let root = std::env::temp_dir().join(format!(
            "fpas-test-process-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).map_err(|error| {
            format!(
                "Error creating isolated test directory `{}`: {error}",
                root.display()
            )
        })?;
        Ok(Self { root })
    }

    fn request(&self) -> PathBuf {
        self.root.join("request.json")
    }

    fn image(&self) -> PathBuf {
        self.root.join("program.fpascp")
    }

    fn ready(&self) -> PathBuf {
        self.root.join("ready")
    }

    fn start(&self) -> PathBuf {
        self.root.join("start")
    }

    fn response(&self) -> PathBuf {
        self.root.join("response.json")
    }

    fn output(&self) -> PathBuf {
        self.root.join("output.txt")
    }

    fn cleanup(&self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Runs one fully compiled program in an OS process that can be forcefully stopped.
pub(super) fn run_with_timeout(
    prepared: PreparedProgram,
    timeout: Duration,
    stderr: &mut dyn Write,
) -> TestOutcome {
    let files = match WorkerFiles::create() {
        Ok(files) => files,
        Err(message) => return worker_failure(stderr, &prepared, &message),
    };
    let result = run_with_files(prepared, timeout, stderr, &files);
    files.cleanup();
    result
}

fn run_with_files(
    prepared: PreparedProgram,
    timeout: Duration,
    stderr: &mut dyn Write,
    files: &WorkerFiles,
) -> TestOutcome {
    if let Err(message) = write_worker_inputs(files, &prepared) {
        return worker_failure(stderr, &prepared, &message);
    }

    let mut child = match spawn_worker(files) {
        Ok(child) => child,
        Err(message) => return worker_failure(stderr, &prepared, &message),
    };
    if let Err(message) = wait_until_ready(&mut child, files) {
        terminate_process_tree(&mut child);
        return worker_failure(stderr, &prepared, &message);
    }

    if let Err(error) = fs::write(files.start(), []) {
        terminate_process_tree(&mut child);
        return worker_failure(
            stderr,
            &prepared,
            &format!("Error starting isolated test process: {error}"),
        );
    }

    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return read_worker_result(files, status.success(), stderr, &prepared);
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                terminate_process_tree(&mut child);
                return timed_out(stderr, &prepared, timeout);
            }
            Err(error) => {
                terminate_process_tree(&mut child);
                return worker_failure(
                    stderr,
                    &prepared,
                    &format!("Error waiting for isolated test process: {error}"),
                );
            }
        }
    }
}

fn write_worker_inputs(files: &WorkerFiles, prepared: &PreparedProgram) -> Result<(), String> {
    let manifest_override = prepared
        .manifest_override
        .as_ref()
        .map(|value| ManifestOverride {
            script: value.script.clone(),
            headless_graph: value.headless_graph,
        });
    let request = WorkerRequest {
        test_path: prepared.test_path.clone(),
        source_paths: prepared.source_paths.as_deref().cloned(),
        script_override: prepared.script_override.clone(),
        manifest_override,
        display: prepared.display.clone(),
        output: prepared.output,
    };
    let request_bytes = serde_json::to_vec(&request)
        .map_err(|error| format!("Error encoding isolated test request: {error}"))?;
    fs::write(files.request(), request_bytes)
        .map_err(|error| format!("Error writing isolated test request: {error}"))?;

    let source_count = prepared
        .source_paths
        .as_ref()
        .map_or(1, |paths| paths.len().max(1));
    let source_paths = (0..source_count)
        .map(|index| format!("source-{index}.fpas"))
        .collect();
    let identity = ProgramIdentity {
        compiler_version: env!("CARGO_PKG_VERSION").to_string(),
        bytecode_version: fpas_bytecode::BYTECODE_VERSION,
        source_hash: Digest::of(b"isolated-test-program"),
        options_hash: Digest::of(b"isolated-test-options"),
        units: Vec::new(),
    };
    let source_hashes = (0..source_count)
        .map(|index| Digest::of(format!("isolated-test-source-{index}")))
        .collect();
    let image = ProgramImage::new(
        identity,
        source_paths,
        source_hashes,
        prepared.executable.clone(),
    )
    .map_err(|error| format!("Error preparing isolated test image: {error}"))?;
    let image_bytes = fpas_program::encode(&image)
        .map_err(|error| format!("Error encoding isolated test image: {error}"))?;
    fs::write(files.image(), image_bytes)
        .map_err(|error| format!("Error writing isolated test image: {error}"))
}

fn spawn_worker(files: &WorkerFiles) -> Result<Child, String> {
    let executable = worker_executable()?;
    let mut command = Command::new(&executable);
    command
        .arg(WORKER_ARGUMENT)
        .arg(&files.root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env("TMP", &files.root)
        .env("TEMP", &files.root)
        .env("TMPDIR", &files.root);
    configure_process_tree(&mut command);
    command.spawn().map_err(|error| {
        format!(
            "Error starting isolated test process `{}`: {error}",
            executable.display()
        )
    })
}

fn worker_executable() -> Result<PathBuf, String> {
    let current = std::env::current_exe()
        .map_err(|error| format!("Error locating the fpas executable: {error}"))?;
    if cfg!(test)
        && current
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|name| name == "deps")
        && let Some(profile) = current.parent().and_then(Path::parent)
    {
        let candidate = profile.join(if cfg!(windows) { "fpas.exe" } else { "fpas" });
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Ok(current)
}

fn wait_until_ready(child: &mut Child, files: &WorkerFiles) -> Result<(), String> {
    loop {
        if files.ready().is_file() {
            return Ok(());
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!(
                    "Isolated test process exited before execution with status {status}."
                ));
            }
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(error) => return Err(format!("Error waiting for isolated test process: {error}")),
        }
    }
}

fn read_worker_result(
    files: &WorkerFiles,
    successful_exit: bool,
    stderr: &mut dyn Write,
    prepared: &PreparedProgram,
) -> TestOutcome {
    if !successful_exit {
        return worker_failure(stderr, prepared, "Isolated test process failed.");
    }
    let output = match fs::read(files.output()) {
        Ok(output) if output.len() <= MAX_CAPTURED_OUTPUT => output,
        Ok(_) => {
            return worker_failure(
                stderr,
                prepared,
                "Isolated test output exceeded the 8 MiB safety limit.",
            );
        }
        Err(error) => {
            return worker_failure(
                stderr,
                prepared,
                &format!("Error reading isolated test output: {error}"),
            );
        }
    };
    let _ = stderr.write_all(&output);
    let response = match fs::read(files.response())
        .map_err(|error| error.to_string())
        .and_then(|bytes| {
            serde_json::from_slice::<WorkerResponse>(&bytes).map_err(|e| e.to_string())
        }) {
        Ok(response) => response,
        Err(error) => {
            return worker_failure(
                stderr,
                prepared,
                &format!("Error reading isolated test result: {error}"),
            );
        }
    };
    response.outcome
}

fn timed_out(stderr: &mut dyn Write, prepared: &PreparedProgram, timeout: Duration) -> TestOutcome {
    if prepared.output.emit_fail_banner() {
        let _ = writeln!(stderr, "  TIMEOUT  {}", prepared.display);
    }
    let _ = writeln!(
        stderr,
        "        test run exceeded {} second timeout.\n  help: Fix an infinite loop or increase `--timeout`.",
        timeout.as_secs()
    );
    TestOutcome::TimedOut
}

fn worker_failure(stderr: &mut dyn Write, prepared: &PreparedProgram, detail: &str) -> TestOutcome {
    if prepared.output.emit_fail_banner() {
        let _ = writeln!(stderr, "  FAIL  {}", prepared.display);
    }
    let _ = writeln!(
        stderr,
        "        test worker failed unexpectedly: {detail}\n  help: Re-run under a debugger or report a compiler/runtime bug."
    );
    TestOutcome::RuntimeError
}

/// Handles the private worker form and returns `None` for all public CLI arguments.
pub(crate) fn run_worker_from_args(args: &[String]) -> Option<i32> {
    if args.first().map(String::as_str) != Some(WORKER_ARGUMENT) {
        return None;
    }
    let Some(root) = args.get(1).map(PathBuf::from) else {
        return Some(2);
    };
    Some(match worker_main(&WorkerFiles { root }) {
        Ok(()) => 0,
        Err(_) => 1,
    })
}

fn worker_main(files: &WorkerFiles) -> Result<(), String> {
    let request = fs::read(files.request())
        .map_err(|error| format!("Error reading worker request: {error}"))
        .and_then(|bytes| {
            serde_json::from_slice::<WorkerRequest>(&bytes)
                .map_err(|error| format!("Error decoding worker request: {error}"))
        })?;
    let image = fs::read(files.image())
        .map_err(|error| format!("Error reading worker image: {error}"))
        .and_then(|bytes| {
            fpas_program::decode(&bytes)
                .map_err(|error| format!("Error decoding worker image: {error}"))
        })?;
    let manifest_override = request
        .manifest_override
        .map(|value| fpas_project::TestFileOverride {
            script: value.script,
            headless_graph: value.headless_graph,
        });
    let prepared = PreparedProgram {
        test_path: request.test_path,
        executable: image.into_executable(),
        source_paths: request.source_paths.map(std::sync::Arc::new),
        script_override: request.script_override,
        manifest_override,
        display: request.display,
        output: request.output,
    };
    let mut output = CappedBuffer::new(MAX_CAPTURED_OUTPUT);
    let outcome = run_prepared_program(prepared, &mut output, || {
        fs::write(files.ready(), [])
            .map_err(|error| format!("Error signaling worker readiness: {error}"))?;
        while !files.start().is_file() {
            thread::sleep(POLL_INTERVAL);
        }
        Ok(())
    })?;
    if output.overflowed() {
        return Err("Isolated test output exceeded the 8 MiB safety limit.".to_string());
    }
    fs::write(files.output(), output.into_inner())
        .map_err(|error| format!("Error writing worker output: {error}"))?;
    let response = serde_json::to_vec(&WorkerResponse { outcome })
        .map_err(|error| format!("Error encoding worker result: {error}"))?;
    fs::write(files.response(), response)
        .map_err(|error| format!("Error writing worker result: {error}"))
}

fn configure_process_tree(command: &mut Command) {
    #[cfg(unix)]
    unix::configure(command);
    #[cfg(windows)]
    windows::configure(command);
}

fn terminate_process_tree(child: &mut Child) {
    #[cfg(unix)]
    unix::terminate(child);
    #[cfg(windows)]
    windows::terminate(child);
    #[cfg(not(any(unix, windows)))]
    {
        let _ = child.kill();
        let _ = child.wait();
    }
}
