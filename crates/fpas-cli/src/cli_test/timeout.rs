//! Wall-clock timeout wrapper for single test VM runs.
//!
//! **Documentation:** [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md)

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use fpas_vm::{VmError, VmShutdownHandle};

const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

/// Result of executing one VM run under an optional wall-clock limit.
pub(super) enum VmRunResult {
    Completed(Result<(), VmError>),
    TimedOut,
}

/// Runs `run` on a worker thread and aborts cooperatively when `timeout` elapses.
pub(super) fn run_with_timeout(
    shutdown: VmShutdownHandle,
    timeout: Duration,
    run: impl FnOnce() -> Result<(), VmError> + Send + 'static,
) -> VmRunResult {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = tx.send(run());
    });

    match rx.recv_timeout(timeout) {
        Ok(result) => {
            let _ = handle.join();
            VmRunResult::Completed(result)
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
