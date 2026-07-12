//! Wall-clock timeout wrapper for single test VM runs.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use fpas_std::UploadedFrame;
use fpas_vm::{VmError, VmShutdownHandle};

const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

/// Captured result and output from one VM execution.
pub(super) struct VmExecution {
    pub result: Result<(), VmError>,
    pub stdout_lines: Vec<String>,
    pub headless_frame: Option<UploadedFrame>,
    pub skipped: bool,
}

/// Result of executing one VM run under an optional wall-clock limit.
pub(super) enum VmRunResult {
    Completed(VmExecution),
    TimedOut,
    WorkerFailed,
}

/// Runs `run` on a worker thread and aborts cooperatively when `timeout` elapses.
pub(super) fn run_with_timeout(
    shutdown: VmShutdownHandle,
    timeout: Duration,
    run: impl FnOnce() -> VmExecution + Send + 'static,
) -> VmRunResult {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = tx.send(run());
    });

    match rx.recv_timeout(timeout) {
        Ok(execution) => {
            let _ = handle.join();
            VmRunResult::Completed(execution)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            shutdown.request_cooperative_shutdown();
            let _ = rx.recv_timeout(SHUTDOWN_GRACE);
            let _ = handle.join();
            VmRunResult::TimedOut
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            let _ = handle.join();
            VmRunResult::WorkerFailed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fpas_compiler::compile_all;
    use fpas_parser::parse;

    #[test]
    fn disconnected_worker_is_not_reported_as_timeout() {
        let (program, _) = parse("program P; begin end.");
        let chunk = compile_all(&program).expect("empty program must compile");
        let vm = fpas_vm::Vm::new(chunk);
        let shutdown = vm.shutdown_handle();
        let result = run_with_timeout(shutdown, Duration::from_secs(60), move || {
            panic!("worker panic");
        });
        assert!(matches!(result, VmRunResult::WorkerFailed));
    }
}
