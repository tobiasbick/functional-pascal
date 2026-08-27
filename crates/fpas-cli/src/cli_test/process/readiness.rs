//! Waits for an isolated test worker to become ready.

use std::io;
use std::path::Path;
use std::process::{Child, ExitStatus};
use std::thread;
use std::time::Instant;

use super::POLL_INTERVAL;

/// State observed while waiting for an isolated worker's execution gate.
pub(super) enum WaitOutcome {
    Ready,
    Exited(ExitStatus),
    TimedOut,
}

/// Waits until the worker is ready, exits, or exhausts the shared deadline.
pub(super) fn wait_until_ready(
    child: &mut Child,
    ready_path: &Path,
    deadline: Instant,
) -> io::Result<WaitOutcome> {
    loop {
        if ready_path.is_file() {
            return Ok(WaitOutcome::Ready);
        }
        match child.try_wait()? {
            Some(status) => return Ok(WaitOutcome::Exited(status)),
            None => {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Ok(WaitOutcome::TimedOut);
                }
                thread::sleep(POLL_INTERVAL.min(remaining));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};
    use std::time::Duration;

    use super::*;
    use crate::test_support::create_temp_dir;

    const NEVER_READY_CHILD: &str = "FPAS_TEST_NEVER_READY_CHILD";

    #[test]
    fn never_ready_child_fixture() {
        if std::env::var_os(NEVER_READY_CHILD).is_some() {
            thread::sleep(Duration::from_secs(1));
        }
    }

    #[test]
    fn wait_until_ready_times_out_before_a_live_child_exits() {
        let directory = create_temp_dir("test-worker-never-ready");
        let ready_path = directory.join("ready");
        let mut child = Command::new(std::env::current_exe().expect("test executable must exist"))
            .args([
                "--exact",
                "cli_test::process::readiness::tests::never_ready_child_fixture",
            ])
            .env(NEVER_READY_CHILD, "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("child fixture must start");
        let started = Instant::now();

        let outcome =
            wait_until_ready(&mut child, &ready_path, started + Duration::from_millis(50))
                .expect("readiness wait must not fail");
        let elapsed = started.elapsed();
        let _ = child.kill();
        let _ = child.wait();
        std::fs::remove_dir_all(directory).expect("temp directory must be removed");

        assert!(matches!(outcome, WaitOutcome::TimedOut));
        assert!(elapsed < Duration::from_millis(500), "elapsed: {elapsed:?}");
    }
}
