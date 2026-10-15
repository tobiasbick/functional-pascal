//! Terminal ownership, retry lifetime, and secondary-cause catalog regressions.

use super::*;
use crate::vm::{TaskResultState, hosted::HostedState};
use std::{
    io::{self, Write},
    sync::mpsc,
    time::{Duration, Instant},
};

struct Barrier {
    entered: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
    fail: bool,
}
impl Write for Barrier {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.entered.send(()).unwrap();
        self.release.recv_timeout(Duration::from_secs(4)).unwrap();
        if self.fail {
            return Err(io::Error::other("body second"));
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn barrier(vm: &mut Vm, fail: bool) -> (mpsc::Receiver<()>, mpsc::Sender<()>) {
    let (entered, receipt) = mpsc::channel();
    let (release, released) = mpsc::channel();
    vm.hosted = Arc::new(HostedState::new(
        fpas_std::Console::with_writer(Box::new(Barrier {
            entered,
            release: released,
            fail,
        })),
        vec![],
    ));
    (receipt, release)
}

fn wait_for_terminal(scheduler: &crate::vm::TaskScheduler, id: u64) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !scheduler.with_capture_results(|results| {
        results
            .get(&id)
            .is_some_and(|state| !matches!(state, TaskResultState::Pending))
    }) {
        assert!(Instant::now() < deadline, "task {id} did not finish");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn disposed_aborted_worker_is_cancelled_without_waiting_for_a_dead_peer() {
    let mut vm = vm(
        "program Abort; uses Std.Console as Console; procedure Barrier(); begin Console.WriteText('entered'); while true do null; end while; end procedure; procedure Fail(); begin panic('root'); end procedure; begin end.",
    );
    let output = observe(&mut vm);
    let (entered, release) = barrier(&mut vm, false);
    let id = enqueue(&vm, "Barrier", vec![]);
    let mut pool = start_pool(&vm);
    entered.recv_timeout(Duration::from_secs(2)).unwrap();
    let mut root = worker(&vm, "Fail", vec![]);
    let capture = Arc::clone(vm.scheduler.failure_capture.get().unwrap());
    capture.start_root(&mut root);
    let error = root.run_task().unwrap_err();
    vm.scheduler.fail(error.clone());
    release.send(()).unwrap();
    pool.thread.take().unwrap().join().unwrap();
    vm.notify_failure(&error);
    let output = output.lock().unwrap();
    let catalog = output[0].task_catalog.as_ref().unwrap();
    let task = catalog.tasks.iter().find(|task| task.id == id).unwrap();
    assert_eq!(task.state, "cancelled");
    assert_eq!(task.inspection, "not_applicable");
    assert!(task.snapshot.is_none());
    assert_ne!(task.reason, Some("collection_deadline"));
    drop(output);
    capture.finish_catalog();
}

#[test]
fn completed_supervised_retry_does_not_retain_a_handled_child_propagation_stack() {
    let mut vm = vm(
        "program Retry; uses Std.Tasks as Tasks; procedure Barrier(); begin null; end procedure; function Work(Attempt:integer):integer; begin if Attempt=0 then panic('retry'); end if; return 42; end function; procedure Launch(Attempts:channel of integer; Hold:channel of integer); begin const Group:=Tasks.CreateTaskGroup(); discard Tasks.StartSupervisedTask(Group, function(Token:Tasks.CancellationToken):integer begin const Attempt:=Tasks.Receive(Attempts).Unwrap(); scope const Child:=go Work(Attempt); return Tasks.Wait(Child); end scope; end function, 1, 0); discard Tasks.Receive(Hold); end procedure; begin panic('unrelated root'); end.",
    );
    vm.pool_size = 0;
    let output = observe(&mut vm);
    let attempts = vm.hosted.channels.create(2).unwrap();
    let hold = vm.hosted.channels.create(1).unwrap();
    for attempt in [0, 1] {
        vm.hosted
            .channels
            .send(attempts, Value::Integer(attempt), false, None)
            .unwrap();
    }
    enqueue(
        &vm,
        "Launch",
        vec![Value::OpaqueHandle(attempts), Value::OpaqueHandle(hold)],
    );
    let timer = start_timer(&vm);
    let pool = start_pool(&vm);
    wait_for_terminal(&vm.scheduler, 2);
    vm.scheduler.with_capture_results(|results| {
        assert!(matches!(results.get(&2), Some(TaskResultState::Value(value)) if matches!(**value, Value::Integer(42))));
    });
    vm.run().unwrap_err();
    let output = output.lock().unwrap();
    let catalog = output[0].task_catalog.as_ref().unwrap();
    let task = catalog.tasks.iter().find(|task| task.id == 2).unwrap();
    assert_eq!(task.state, "completed");
    assert_eq!(task.inspection, "not_applicable");
    assert!(
        task.snapshot.is_none(),
        "handled attempt frames survived retry"
    );
    drop(output);
    drop(timer);
    drop(pool);
}

#[test]
fn root_secondary_cause_populates_the_main_catalog_entry() {
    let mut vm = vm(
        "program Secondary; uses Std.Tasks as Tasks, Std.Console as Console; var Proceed:channel of integer:=Tasks.CreateChannel(1); function Work(Ready:channel of integer):integer; begin discard Tasks.Receive(Ready); const ChildLocal:=37; panic('child first'); end function; begin scope discard go Work(Proceed); const RootLocal:=19; Console.WriteText('root barrier'); end scope; end.",
    );
    vm.pool_size = 1;
    let output = observe(&mut vm);
    let (entered, release) = barrier(&mut vm, true);
    let scheduler = Arc::clone(&vm.scheduler);
    let globals = Arc::clone(&vm.globals);
    let hosted = Arc::clone(&vm.hosted);
    let thread = std::thread::spawn(move || vm.run().unwrap_err());
    entered.recv_timeout(Duration::from_secs(2)).unwrap();
    let Some(Value::OpaqueHandle(proceed)) = globals.read().unwrap()[0].clone() else {
        panic!("the program must initialize its channel before entering the barrier");
    };
    hosted
        .channels
        .send(proceed, Value::Integer(1), false, None)
        .unwrap();
    wait_for_terminal(&scheduler, 1);
    release.send(()).unwrap();
    assert_eq!(thread.join().unwrap().message, "panic: child first");
    let output = output.lock().unwrap();
    let data = &output[0];
    assert_eq!(data.cause.task_id, Some(1));
    let root_error = data
        .related_errors
        .iter()
        .find(|error| error.cause.task_id == Some(0))
        .expect("the real body panic remains a structured secondary");
    let task = data
        .task_catalog
        .as_ref()
        .unwrap()
        .tasks
        .iter()
        .find(|task| task.id == 0)
        .unwrap();
    assert_eq!(task.observation, "fault");
    assert!(Arc::ptr_eq(
        task.snapshot.as_ref().unwrap(),
        &root_error.cause
    ));
    assert_eq!(
        scalar(&task.snapshot.as_ref().unwrap().frames[0], "RootLocal"),
        &Some(crate::vm::FailureScalar::Integer("19".into()))
    );
}

#[test]
fn omitted_released_cause_identities_are_included_in_catalog_totals() {
    let mut vm =
        vm("program Causes; procedure Fail(); begin panic('cause'); end procedure; begin end.");
    let output = observe(&mut vm);
    let mut errors = Vec::new();
    for id in 1..=1030 {
        let mut cause = worker(&vm, "Fail", vec![]);
        cause.task_id = id;
        errors.push(cause.run_task().unwrap_err());
    }
    let mut primary = errors.remove(0);
    super::super::super::attach_related(&mut primary, &errors);
    vm.notify_failure(&primary);
    let output = output.lock().unwrap();
    let data = &output[0];
    assert_eq!(data.related_errors.len(), 1024);
    let catalog = data.task_catalog.as_ref().unwrap();
    assert_eq!(
        catalog.total_tasks, 1026,
        "main, primary, and all known causes"
    );
    assert_eq!(catalog.tasks.len(), 1024);
    assert_eq!(catalog.omitted_tasks, 2);
}

#[test]
fn converted_pool_panic_keeps_completed_root_globals_and_marks_the_dead_peer_failed() {
    struct Panics;
    impl Write for Panics {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            panic!("host writer failed");
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut vm = vm(
        "program HostPanic; uses Std.Console as Console; var Total:integer:=7; procedure Background(); begin Console.WriteText('host'); end procedure; begin Total:=31; end.",
    );
    vm.pool_size = 1;
    let output = observe(&mut vm);
    vm.hosted = Arc::new(HostedState::new(
        fpas_std::Console::with_writer(Box::new(Panics)),
        vec![],
    ));
    let id = enqueue(&vm, "Background", vec![]);
    assert!(vm.run().unwrap_err().message.contains("worker panicked"));
    let output = output.lock().unwrap();
    let catalog = output[0].task_catalog.as_ref().unwrap();
    assert_eq!(catalog.globals_status, "available");
    assert_eq!(
        catalog.globals.as_ref().unwrap().bindings[0].value.scalar,
        Some(crate::vm::FailureScalar::Integer("31".into()))
    );
    let root = catalog.tasks.iter().find(|task| task.id == 0).unwrap();
    assert_eq!(root.state, "completed");
    assert_eq!(root.inspection, "not_applicable");
    let peer = catalog.tasks.iter().find(|task| task.id == id).unwrap();
    assert_eq!(peer.state, "failed");
    assert_eq!(peer.reason, Some("no_live_frame"));
}
