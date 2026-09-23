//! Kill failures must retain diagnostics and must never perform an unbounded wait.
use super::*;
use process_wrap::std::ChildWrapper;
use std::process::{ChildStderr, ChildStdout, ExitStatus};

#[derive(Debug)]
struct FailingKill {
    child: Box<dyn ChildWrapper>,
    fail_fallback: bool,
}

impl ChildWrapper for FailingKill {
    fn inner(&self) -> &dyn ChildWrapper {
        self.child.as_ref()
    }
    fn inner_mut(&mut self) -> &mut dyn ChildWrapper {
        if self.fail_fallback {
            self
        } else {
            self.child.as_mut()
        }
    }
    fn into_inner(self: Box<Self>) -> Box<dyn ChildWrapper> {
        self
    }
    fn stdout(&mut self) -> &mut Option<ChildStdout> {
        self.child.stdout()
    }
    fn stderr(&mut self) -> &mut Option<ChildStderr> {
        self.child.stderr()
    }
    fn start_kill(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected tree failure"))
    }
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }
    fn wait(&mut self) -> io::Result<ExitStatus> {
        panic!("cleanup must use bounded polling, even after a kill error")
    }
}

impl Drop for FailingKill {
    fn drop(&mut self) {
        // The injected failure ends here; always clean up the real fixture tree.
        let _ = super::super::termination::terminate_process_tree(self.child.as_mut());
    }
}

#[test]
fn failed_tree_kill_uses_fallback_and_returns() -> Result<(), Box<dyn Error>> {
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args([
            "--exact",
            "suite::runner::tests::timeout_child_fixture",
            "--nocapture",
        ])
        .env("FPAS_BENCH_TIMEOUT_FIXTURE", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = FailingKill {
        child: spawn_benchmark(command)?,
        fail_fallback: false,
    };
    let started = Instant::now();
    let error = wait_for_output(Box::new(child), Duration::from_millis(100), "fallback")
        .expect_err("fixture times out");
    assert!(error.contains("injected tree failure"), "{error}");
    assert!(!error.contains("failed to reap"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(5));
    Ok(())
}

#[test]
fn failed_kills_with_live_descendant_retain_reap_and_pipe_errors() -> Result<(), Box<dyn Error>> {
    let ready = std::env::temp_dir().join(format!("fpas-bench-failed-kill-{}", std::process::id()));
    let _ = fs::remove_file(&ready);
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args([
            "--exact",
            "suite::runner::tests::timeout_tree_parent_fixture",
            "--nocapture",
        ])
        .env("FPAS_BENCH_TIMEOUT_TREE_READY", &ready)
        .env("FPAS_BENCH_TIMEOUT_TREE_HOLD", "30")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = FailingKill {
        child: spawn_benchmark(command)?,
        fail_fallback: true,
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ready.is_file() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        ready.is_file(),
        "grandchild must hold the inherited pipes before timing"
    );
    let started = Instant::now();
    let error = wait_for_output(Box::new(child), Duration::from_millis(100), "failed-kills")
        .expect_err("fixture times out");
    let _ = fs::remove_file(ready);
    for expected in [
        "injected tree failure",
        "direct kill failed",
        "failed to reap process",
        "stdout remained open",
        "stderr remained open",
    ] {
        assert!(error.contains(expected), "missing {expected}: {error}");
    }
    assert!(started.elapsed() < Duration::from_secs(10));
    Ok(())
}
