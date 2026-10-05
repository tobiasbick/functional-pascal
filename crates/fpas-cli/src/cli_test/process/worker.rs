//! Worker side of the isolated test-process protocol.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../../docs/pascal/std/testing/test.md)

use std::fs;
use std::path::PathBuf;
use std::thread;

use super::output::CappedBuffer;
use super::{
    MAX_CAPTURED_OUTPUT, POLL_INTERVAL, WORKER_ARGUMENT, WorkerFiles, WorkerRequest, WorkerResponse,
};
use crate::cli_output::Reporter;
use crate::cli_test::run::program::{PreparedProgram, run_prepared_program};

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
        });
    let prepared = PreparedProgram {
        test_path: request.test_path,
        executable: image.into_executable(),
        source_paths: request.source_paths.map(std::sync::Arc::new),
        script_override: request.script_override,
        manifest_override,
        display: request.display,
        output: request.output,
        show_output: request.show_output,
        scratch_dir: request.scratch_dir,
    };
    let mut output = CappedBuffer::new(MAX_CAPTURED_OUTPUT);
    let mut program_output = Vec::new();
    let mut reporter =
        Reporter::new(request.diagnostics, &mut output).with_program_output(&mut program_output);
    let outcome = run_prepared_program(prepared, &mut reporter, || {
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
    fs::write(files.program_output(), program_output)
        .map_err(|error| format!("Error writing worker program output: {error}"))?;
    let response = serde_json::to_vec(&WorkerResponse { outcome })
        .map_err(|error| format!("Error encoding worker result: {error}"))?;
    fs::write(files.response(), response)
        .map_err(|error| format!("Error writing worker result: {error}"))
}
