//! Channel and queue barriers establish real saved and running states before the catalog cut.

use super::*;
use crate::vm::{FailureScalar, TaskResultPoll, channels::ReceiveState, hosted::HostedState};
use std::{
    io::{self, Write},
    sync::mpsc,
    time::Duration,
};

#[test]
fn catalog_captures_saved_running_and_retained_tasks_without_executing_or_consuming_them() {
    let source = "program Catalog; uses Std.Tasks as Tasks, Std.Time as Time, Std.Console as Console;\nprocedure Sleeping(); begin const SleepLocal:=11; Time.Sleep(60000); end procedure;\nprocedure Waiting(Inbox:channel of integer); begin const WaitLocal:=13; discard Tasks.Receive(Inbox); end procedure;\nfunction Completed():integer; begin return 42; end function;\nprocedure Failed(); begin const FailedLocal:=17; panic('retained'); end procedure;\nprocedure Barrier(); begin Console.WriteText('barrier'); end procedure;\nprocedure Running(); begin const Live:=29; Console.WriteText('running'); while true do null; end while; end procedure;\nprocedure Queued(Inbox:channel of integer); begin panic('queued code must not execute'); end procedure;\nbegin panic('root'); end.";
    let mut vm = vm(source);
    vm.pool_size = 0;
    let output = observe(&mut vm);
    let (ready_tx, ready_rx) = mpsc::channel();
    struct Receipt(mpsc::Sender<String>);
    impl Write for Receipt {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0
                .send(String::from_utf8(bytes.to_vec()).unwrap())
                .unwrap();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    vm.hosted = Arc::new(HostedState::new(
        fpas_std::Console::with_writer(Box::new(Receipt(ready_tx))),
        vec![],
    ));
    let empty = vm.hosted.channels.create(1).unwrap();
    let sleeping = enqueue(&vm, "Sleeping", vec![]);
    let waiting = enqueue(&vm, "Waiting", vec![Value::OpaqueHandle(empty)]);
    let completed = enqueue(&vm, "Completed", vec![]);
    let released = enqueue(&vm, "Completed", vec![]);
    let failed = enqueue(&vm, "Failed", vec![]);
    let group = vm
        .scheduler
        .groups
        .create(9000, || vm.hosted.cancellations.create_owned())
        .unwrap();
    vm.scheduler.groups.enroll(group, 9000, failed).unwrap();
    let mut cancelled_worker = worker(&vm, "Completed", vec![]);
    let cancelled = vm.scheduler.alloc_id();
    cancelled_worker.task_id = cancelled;
    cancelled_worker.retain_result = true;
    let (scope, _) = vm.scheduler.scopes.open();
    cancelled_worker.owner_scope = vm.scheduler.scopes.enroll(scope, cancelled);
    vm.scheduler.scopes.cancel(scope);
    vm.scheduler.register_result(cancelled);
    vm.scheduler.enqueue(cancelled_worker.take_task_state());
    enqueue(&vm, "Barrier", vec![]);
    let running = enqueue(&vm, "Running", vec![]);
    let guard = start_pool(&vm);
    assert_eq!(
        ready_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        "barrier"
    );
    assert_eq!(
        ready_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        "running"
    );
    assert!(matches!(
        vm.scheduler.poll_result(released),
        TaskResultPoll::Available(Value::Integer(42))
    ));
    assert_eq!(vm.run().unwrap_err().message, "panic: root");
    let output = output.lock().unwrap();
    let data = &output[0];
    let catalog = data.task_catalog.as_ref().unwrap();
    let ids: Vec<_> = catalog.tasks.iter().map(|task| task.id).collect();
    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(!ids.contains(&released));
    for (id, state, observation, local, expected) in [
        (sleeping, "sleeping", "failure_handling", "SleepLocal", 11),
        (waiting, "waiting", "failure_handling", "WaitLocal", 13),
        (running, "running", "failure_handling", "Live", 29),
        (failed, "failed", "fault", "FailedLocal", 17),
    ] {
        let task = catalog.tasks.iter().find(|task| task.id == id).unwrap();
        if id == running {
            assert!(matches!(task.state, "running" | "runnable"));
        } else {
            assert_eq!(task.state, state, "{task:?}");
        }
        assert_eq!(task.observation, observation);
        assert_eq!(task.inspection, "available", "{task:?}");
        assert_eq!(
            scalar(&task.snapshot.as_ref().unwrap().frames[0], local),
            &Some(FailureScalar::Integer(expected.to_string()))
        );
    }
    for (id, state) in [(completed, "completed"), (cancelled, "cancelled")] {
        let task = catalog.tasks.iter().find(|task| task.id == id).unwrap();
        assert_eq!(task.state, state);
        assert_eq!(task.inspection, "not_applicable");
        assert!(task.snapshot.is_none());
    }
    assert!(matches!(
        vm.scheduler.poll_result(completed),
        TaskResultPoll::Available(Value::Integer(42))
    ));
    drop(output);
    drop(guard);
}

#[test]
fn queued_inspection_runs_no_code_and_consumes_no_channel_or_task_result() {
    let mut vm = vm(
        "program Queued; procedure Pending(Inbox:channel of integer); begin panic('inspection dispatched queued code'); end procedure; begin panic('root'); end.",
    );
    vm.pool_size = 0;
    let output = observe(&mut vm);
    let channel = vm.hosted.channels.create(1).unwrap();
    vm.hosted
        .channels
        .send(channel, Value::Integer(71), false, None)
        .unwrap();
    let queued = enqueue(&vm, "Pending", vec![Value::OpaqueHandle(channel)]);
    assert_eq!(vm.run().unwrap_err().message, "panic: root");
    let output = output.lock().unwrap();
    let catalog = output[0].task_catalog.as_ref().unwrap();
    let task = catalog.tasks.iter().find(|task| task.id == queued).unwrap();
    assert_eq!(task.state, "runnable");
    assert_eq!(task.inspection, "available");
    assert_eq!(
        task.snapshot.as_ref().unwrap().frames[0].bindings[0]
            .value
            .status,
        "uninitialized"
    );
    assert!(matches!(
        vm.scheduler.poll_result(queued),
        TaskResultPoll::Pending
    ));
    assert!(matches!(
        vm.hosted.channels.receive(channel, false, None).unwrap(),
        ReceiveState::Received(Value::Integer(71))
    ));
}

#[test]
fn catalog_cut_survives_spawn_wait_completion_and_release_handoffs() {
    let source = "program Handoff; uses Std.Tasks as Tasks, Std.Console as Console; function Work(Item:integer):integer; begin return Item*Item; end function; procedure Barrier(); begin Console.WriteText('ready'); for I:integer:=1 to 96 do scope const Job:=go Work(I); discard Tasks.Wait(Job); end scope; end for; end procedure; begin panic('root'); end.";
    let image = Arc::clone(&vm(source).executable);
    for _ in 0..16 {
        let mut vm = Vm::from_shared(Arc::clone(&image));
        vm.pool_size = 0;
        let output = observe(&mut vm);
        let (ready_tx, ready_rx) = mpsc::channel();
        struct Ready(mpsc::Sender<()>);
        impl Write for Ready {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.0.send(()).unwrap();
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        vm.hosted = Arc::new(HostedState::new(
            fpas_std::Console::with_writer(Box::new(Ready(ready_tx))),
            vec![],
        ));
        let parent = enqueue(&vm, "Barrier", vec![]);
        let timer = start_timer(&vm);
        let pool = start_pool(&vm);
        ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(vm.run().unwrap_err().message, "panic: root");
        let output = output.lock().unwrap();
        let catalog = output[0].task_catalog.as_ref().unwrap();
        assert_eq!(catalog.omitted_tasks, 0);
        assert_eq!(catalog.tasks.len(), catalog.total_tasks);
        assert!(catalog.tasks.iter().any(|task| task.id == parent));
        assert!(catalog.tasks.windows(2).all(|pair| pair[0].id < pair[1].id));
        for task in &catalog.tasks {
            if let Some(snapshot) = &task.snapshot {
                assert_eq!(snapshot.task_id, Some(task.id));
            }
        }
        drop(output);
        drop(timer);
        drop(pool);
    }
}

#[test]
fn globals_use_actual_local_storage_and_survive_server_finalization_errors() {
    for body in [
        "panic('fault');",
        "discard Server.CreateLifetime(0,false).Unwrap();",
    ] {
        let mut vm = vm(&format!(
            "program Globals; uses Std.Server as Server; var Total:integer:=7; begin Total:=31; {body} end."
        ));
        vm.pool_size = 0;
        vm.use_local_globals = true;
        let output = observe(&mut vm);
        vm.run().unwrap_err();
        assert!(
            vm.globals.read().unwrap()[0].is_none(),
            "fixture must use the local storage path"
        );
        let output = output.lock().unwrap();
        let catalog = output[0].task_catalog.as_ref().unwrap();
        assert_eq!(catalog.globals_status, "available");
        let global = &catalog.globals.as_ref().unwrap().bindings[0];
        assert_eq!(global.name.to_ascii_lowercase(), "total");
        assert_eq!(
            global.value.scalar,
            Some(FailureScalar::Integer("31".into()))
        );
    }
}

#[test]
fn expired_sleep_is_runnable_when_timer_publishes_it_without_dispatch() {
    let mut vm = vm(
        "program Due; procedure Pending(); begin panic('timer task must not execute'); end procedure; begin panic('root'); end.",
    );
    vm.pool_size = 0;
    let output = observe(&mut vm);
    let signal = crate::vm::shared::wakeups::WakeSignal::new();
    let _registration = vm.scheduler.subscribe(&signal);
    let mut sleeper = worker(&vm, "Pending", vec![]);
    sleeper.task_id = vm.scheduler.alloc_id();
    sleeper.retain_result = true;
    let id = sleeper.task_id;
    vm.scheduler.register_result(id);
    vm.scheduler.schedule(sleeper.take_task_state(), 0);
    let timer = start_timer(&vm);
    assert!(
        signal.wait(Duration::from_secs(2)),
        "timer must publish runnable work"
    );
    assert_eq!(vm.run().unwrap_err().message, "panic: root");
    let output = output.lock().unwrap();
    let catalog = output[0].task_catalog.as_ref().unwrap();
    let task = catalog.tasks.iter().find(|task| task.id == id).unwrap();
    assert_eq!(task.state, "runnable");
    assert_eq!(task.inspection, "available");
    assert!(matches!(
        vm.scheduler.poll_result(id),
        TaskResultPoll::Pending
    ));
    drop(output);
    drop(timer);
}
