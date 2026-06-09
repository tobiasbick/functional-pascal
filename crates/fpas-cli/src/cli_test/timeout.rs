//! Wall-clock timeout wrapper for single test VM runs.
//!
//! **Documentation:** [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md)

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
    pub screen_lines: Vec<String>,
    pub headless_frame: Option<UploadedFrame>,
}

/// Result of executing one VM run under an optional wall-clock limit.
pub(super) enum VmRunResult {
    Completed(VmExecution),
    TimedOut,
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
            VmRunResult::TimedOut
        }
    }
}
