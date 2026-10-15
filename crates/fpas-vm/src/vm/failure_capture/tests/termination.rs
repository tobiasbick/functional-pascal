//! Terminal notification never substitutes capture failures for program failures.

use super::*;
use fpas_diagnostics::codes::{RUNTIME_PROGRAM_PANIC, RUNTIME_VM_SHUTDOWN};

#[test]
fn observer_panic_preserves_program_error_and_owned_child_cleanup() {
    let mut vm = vm(
        "program Observer; uses Std.Tasks as Tasks; begin const Group := Tasks.CreateTaskGroup(); discard Tasks.StartTaskInGroup(Group, procedure(Token: Tasks.CancellationToken) begin end procedure); panic('program failure'); end.",
    );
    assert!(vm.set_failure_observer(|_| panic!("observer failed")));
    let error = vm.run().unwrap_err();
    assert_eq!(error.code, RUNTIME_PROGRAM_PANIC);
    assert_eq!(error.message, "panic: program failure");
    assert!(
        vm.scheduler.groups.owned_groups(0).is_empty(),
        "capture failure bypassed cleanup"
    );
}

#[test]
fn escaping_shutdown_and_no_live_frame_are_explicit() {
    let mut cancelled = vm("program Cancelled; begin end.");
    let output = observe(&mut cancelled);
    cancelled.shutdown_handle().shutdown();
    let error = cancelled.run().unwrap_err();
    assert_eq!(error.code, RUNTIME_VM_SHUTDOWN);
    let output = output.lock().unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].cause.task_id, Some(0));
    assert!(!output[0].cause.frames.is_empty());

    let mut reused = vm("program Reused; begin end.");
    let output = observe(&mut reused);
    reused.run().unwrap();
    let error = reused.run().unwrap_err();
    assert_eq!(error.code, RUNTIME_VM_SHUTDOWN);
    let output = output.lock().unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].cause.task_id, None);
    assert_eq!(output[0].cause.reason, Some("no_live_frame"));
    assert!(output[0].cause.frames.is_empty());
}

#[test]
fn terminal_observation_arrives_before_a_remaining_native_worker_join() {
    use crate::vm::{hosted::HostedState, worker::Worker};
    use fpas_bytecode::FunctionId;
    use std::{
        io::{self, Write},
        sync::mpsc,
        time::Duration,
    };

    struct BlockingWriter {
        entered: mpsc::Sender<()>,
        release: mpsc::Receiver<()>,
    }
    impl Write for BlockingWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let _ = self.entered.send(());
            self.release
                .recv_timeout(Duration::from_secs(5))
                .map_err(io::Error::other)?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    for (body, expected_task) in [
        ("panic('root');", Some(0)),
        ("discard Server.CreateLifetime(0, false).Unwrap();", None),
    ] {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (observed_tx, observed_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let mut vm = vm(&format!(
            "program Early; uses Std.Console as Console, Std.Server as Server; procedure Background(); begin Console.WriteText('blocked'); end procedure; begin {body} end."
        ));
        vm.pool_size = 1;
        vm.hosted = Arc::new(HostedState::new(
            fpas_std::Console::with_writer(Box::new(BlockingWriter {
                entered: entered_tx,
                release: release_rx,
            })),
            vec![],
        ));
        let entered = Mutex::new(entered_rx);
        assert!(vm.set_failure_observer(move |data| {
            entered
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(2))
                .unwrap();
            observed_tx.send(data).unwrap();
        }));
        let target = vm
            .executable
            .executable()
            .functions
            .iter()
            .position(|info| {
                vm.executable.executable().strings.get(info.name).unwrap() == "background"
            })
            .unwrap();
        let mut background = Worker::for_function_with_state(
            Arc::clone(&vm.executable),
            FunctionId::try_from_index(target).unwrap(),
            vec![],
            Arc::clone(&vm.globals),
            vm.layouts.clone().unwrap(),
            Arc::clone(&vm.hosted),
        )
        .unwrap()
        .with_scheduler(Some(Arc::clone(&vm.scheduler)));
        background.task_id = 1;
        // Model remaining hosted/background work, outside the root's owned task scopes.
        vm.scheduler.enqueue(background.take_task_state());
        let runner = std::thread::spawn(move || {
            let result = vm.run();
            done_tx.send(()).unwrap();
            result
        });
        let observation = observed_rx.recv_timeout(Duration::from_secs(3));
        let still_joining = matches!(done_rx.try_recv(), Err(mpsc::TryRecvError::Empty));
        let _ = release_tx.send(());
        let error = runner.join().unwrap().unwrap_err();
        let observation = observation.expect("notification must precede the blocked native join");
        assert!(
            still_joining,
            "ownership cleanup must still join the worker"
        );
        assert_eq!(observation.error.diagnostic, error);
        assert_eq!(observation.cause.task_id, expected_task);
        if expected_task.is_none() {
            assert_eq!(observation.cause.reason, Some("no_live_frame"));
        }
    }
}
