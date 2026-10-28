//! Explicit shutdown releases unused cases even while a synchronous call is blocked.

use super::*;
use crate::execution::vm::Vm;
use crate::execution::vm::worker::{FunctionInputs, Worker};
use std::io::{self, Write};
use std::sync::mpsc;
use std::time::Duration;

struct BlockingWriter {
    started: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
    entered: bool,
}

impl Write for BlockingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !self.entered {
            self.entered = true;
            self.started.send(()).map_err(io::Error::other)?;
            self.release.recv().map_err(io::Error::other)?;
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct ReleaseOnDrop(mpsc::Sender<()>);

impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}

#[test]
fn shutdown_releases_unused_case_and_closes_admission_before_blocked_call_returns() {
    let image = image(
        r#"
program BlockedHostCall;
uses Std.Tasks as Tasks, Std.Console as Console;
function NewCase(): Tasks.WaitCase;
begin
  var Seen := 0;
  return Tasks.TimerCase(0, procedure() begin Seen := Seen + 1; end procedure);
end function;
procedure Block();
begin
  discard NewCase();
  Console.WriteText('blocked');
end procedure;
begin
end.
"#,
    );
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let mut vm = Vm::with_writer(
        (*image).clone(),
        Box::new(BlockingWriter {
            started: started_tx,
            release: release_rx,
            entered: false,
        }),
    );
    let target = function(&vm.executable, "Block");
    let create = function(&vm.executable, "NewCase");
    let executable = Arc::clone(&vm.executable);
    let domain = vm.domain.clone();
    let globals = Arc::clone(&vm.globals);
    let layouts = vm.layouts.clone().expect("runtime layouts");
    let hosted = Arc::clone(&vm.hosted);
    let shutdown = vm.shutdown_handle();
    std::thread::scope(|scope| {
        let running = scope.spawn(move || vm.call(target, vec![]));
        // Unblock the native call even if an assertion fails before the explicit release.
        let release = ReleaseOnDrop(release_tx);
        started_rx
            .recv_timeout(Duration::from_secs(30))
            .expect("call reaches native barrier");
        let captures = domain.live_cells();
        assert_eq!(captures.len(), 1, "unused case retains its mutable capture");
        let captured = captures[0].downgrade();
        drop(captures);
        shutdown.shutdown();
        assert!(
            captured.upgrade().is_none(),
            "shutdown releases the registry's callback capture"
        );
        let refused = Worker::for_function_in_domain(
            executable,
            create,
            FunctionInputs {
                arguments: &[],
                captures: &[],
                domain: crate::MemoryDomain::new(),
            },
            globals,
            layouts,
            hosted,
        )
        .expect("fresh synchronous probe")
        .run()
        .expect_err("session shutdown closes new case admission");
        assert_eq!(refused.code, RUNTIME_INVALID_TASK);
        assert!(
            refused.message.contains("registry is closed"),
            "{refused:?}"
        );
        assert!(
            !running.is_finished(),
            "cleanup precedes the blocked call's return"
        );
        drop(release);
        running
            .join()
            .expect("call thread")
            .expect("native call finishes");
    });
}
