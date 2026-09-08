//! Source validation and channel commitment under competing selections.

use super::*;
use crate::vm::tasks::TaskScheduler;
use fpas_bytecode::{FunctionId, SharedFunction};

fn worker() -> Worker {
    let (program, errors) = fpas_parser::parse("program T; begin end.");
    assert!(errors.is_empty());
    let mut worker = Worker::new(Arc::new(fpas_compiler::compile(&program).unwrap())).unwrap();
    worker.scheduler = Some(Arc::new(TaskScheduler::new()));
    worker
}

#[test]
fn root_selection_leaves_queued_computation_for_pool_workers() {
    let mut worker = worker();
    let scheduler = Arc::clone(worker.scheduler_ref().unwrap());
    let function = SharedFunction::unbound(FunctionId::new(0), "queued computation".into(), vec![]);
    let info = &worker.executable.executable().functions[0];
    scheduler.register_result(1);
    scheduler.enqueue(crate::vm::tasks::TaskState::entry(
        1,
        &function,
        info,
        [],
        true,
    ));
    worker
        .run_selection(wait(vec![CaseSource::Timer(20)]))
        .unwrap();
    assert_eq!(
        scheduler
            .try_dequeue()
            .expect("computation must remain queued")
            .id,
        1
    );
}

fn wait(sources: Vec<CaseSource>) -> SelectionWait {
    SelectionWait {
        cases: sources
            .into_iter()
            .map(|source| WaitCase {
                source,
                callback: SharedFunction::unbound(
                    FunctionId::new(0),
                    "unused callback".into(),
                    vec![],
                ),
            })
            .collect(),
        started: Instant::now(),
        debug_started: 0,
        destination: None,
    }
}

#[test]
fn invalid_later_source_prevents_an_earlier_ready_receive() {
    let worker = worker();
    let queue = worker.hosted.channels.create(1).unwrap();
    worker
        .hosted
        .channels
        .send(queue, Value::Integer(42), false, None)
        .unwrap();
    let selection = wait(vec![CaseSource::Receive(queue), CaseSource::Receive(0)]);
    assert!(worker.selection_probe(&selection).is_err());
    assert!(matches!(
        worker.hosted.channels.receive(queue, false, None).unwrap(),
        ReceiveState::Received(Value::Integer(42))
    ));
}

#[test]
fn task_failure_prevents_channel_commit_and_retains_its_diagnostic() {
    let worker = worker();
    let queue = worker.hosted.channels.create(1).unwrap();
    let scheduler = worker.scheduler_ref().unwrap();
    scheduler.register_result(1);
    let error = worker.selection_error("original child failure");
    scheduler.store_failure(1, error.clone());
    let selection = wait(vec![
        CaseSource::Send(queue, Value::Integer(42)),
        CaseSource::Task(1),
    ]);
    assert_eq!(worker.selection_probe(&selection).unwrap_err(), error);
    assert!(matches!(
        worker.hosted.channels.receive(queue, false, None).unwrap(),
        ReceiveState::Pending
    ));
}

#[test]
fn first_ready_send_commits_without_enqueueing_a_losing_value() {
    let worker = worker();
    let queue = worker.hosted.channels.create(2).unwrap();
    let selection = wait(vec![
        CaseSource::Send(queue, Value::Integer(1)),
        CaseSource::Send(queue, Value::Integer(2)),
    ]);
    assert_eq!(
        worker.selection_probe(&selection).unwrap(),
        Some((0, Some(Value::result_ok(Value::Boolean(true)))))
    );
    assert!(matches!(
        worker.hosted.channels.receive(queue, false, None).unwrap(),
        ReceiveState::Received(Value::Integer(1))
    ));
    assert!(matches!(
        worker.hosted.channels.receive(queue, false, None).unwrap(),
        ReceiveState::Pending
    ));
}

#[test]
fn cancellation_and_timer_obey_input_order_without_consuming_channel_values() {
    let worker = worker();
    let queue = worker.hosted.channels.create(1).unwrap();
    worker
        .hosted
        .channels
        .send(queue, Value::Integer(42), false, None)
        .unwrap();
    let token = worker.hosted.cancellations.create_source();
    worker.hosted.cancellations.cancel(token).unwrap();
    for sources in [
        vec![CaseSource::Cancellation(token), CaseSource::Receive(queue)],
        vec![CaseSource::Timer(0), CaseSource::Cancellation(token)],
    ] {
        assert_eq!(
            worker.selection_probe(&wait(sources)).unwrap(),
            Some((0, None))
        );
    }
    assert!(matches!(
        worker.hosted.channels.receive(queue, false, None).unwrap(),
        ReceiveState::Received(Value::Integer(42))
    ));
}

#[test]
fn competing_selections_deliver_each_buffered_value_exactly_once() {
    let worker = worker();
    let queue = worker.hosted.channels.create(1000).unwrap();
    for value in 0..1000 {
        worker
            .hosted
            .channels
            .send(queue, Value::Integer(value), false, None)
            .unwrap();
    }
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let hosted = Arc::clone(&worker.hosted);
            std::thread::spawn(move || {
                let mut worker = self::worker();
                worker.hosted = hosted;
                (0..250)
                    .map(|_| {
                        let outcome = worker
                            .selection_probe(&wait(vec![
                                CaseSource::Receive(queue),
                                CaseSource::Timer(0),
                            ]))
                            .unwrap();
                        let Some((0, Some(Value::ResultOk(value)))) = outcome else {
                            panic!("receive must win: {outcome:?}");
                        };
                        let Value::Integer(value) = value.as_ref() else {
                            panic!("integer payload");
                        };
                        *value
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    let mut values: Vec<_> = threads
        .into_iter()
        .flat_map(|thread| thread.join().unwrap())
        .collect();
    values.sort_unstable();
    assert_eq!(values, (0..1000).collect::<Vec<_>>());
    assert!(matches!(
        worker.hosted.channels.receive(queue, false, None).unwrap(),
        ReceiveState::Pending
    ));
}

#[test]
fn callback_panic_keeps_the_committed_send_and_losing_captures_are_already_released() {
    use std::sync::Mutex;
    let (program, errors) = fpas_parser::parse(
        r#"program CallbackFailure;
uses Std.Task;
procedure FailSelected(Outcome: result of boolean, string);
begin panic('selected callback failed') end;
begin FailSelected(Ok(true)) end."#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let executable = fpas_compiler::compile(&program).unwrap();
    let image = executable.executable();
    let callback = image
        .functions
        .iter()
        .position(|info| {
            image.strings.get(info.name).is_some_and(|name| {
                name.rsplit('.')
                    .next()
                    .unwrap_or(name)
                    .eq_ignore_ascii_case("FailSelected")
            })
        })
        .expect("callback target");
    let mut worker = Worker::new(Arc::new(executable)).unwrap();
    worker.scheduler = Some(Arc::new(TaskScheduler::new()));
    let queue = worker.hosted.channels.create(1).unwrap();
    let captured = Arc::new(Mutex::new(Value::Integer(9)));
    let weak = Arc::downgrade(&captured);
    let mut selection = wait(vec![
        CaseSource::Send(queue, Value::Integer(42)),
        CaseSource::Timer(0),
    ]);
    selection.cases[0].callback = SharedFunction::unbound(
        FunctionId::new(callback as u16),
        "FailSelected".into(),
        vec![],
    );
    selection.cases[1].callback = SharedFunction::task_owned(
        FunctionId::new(0),
        "loser".into(),
        vec![Value::Cell(captured)],
        0,
    );
    worker.run_selection(selection).unwrap();
    assert!(
        weak.upgrade().is_none(),
        "losing captures must be released before callback execution"
    );
    let error = worker.run_in_place().expect_err("callback panic");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    assert!(
        error.message.contains("selected callback failed"),
        "{error:?}"
    );
    assert!(matches!(
        worker.hosted.channels.receive(queue, false, None).unwrap(),
        ReceiveState::Received(Value::Integer(42))
    ));
}

#[test]
fn abandoning_a_suspended_debug_selection_releases_its_owned_cases() {
    use std::sync::Mutex;
    let mut worker = worker();
    worker.debug_tasks = true;
    worker.task_clock = Some(Arc::new(TaskClock::manual()));
    let captured = Arc::new(Mutex::new(Value::Integer(9)));
    let weak = Arc::downgrade(&captured);
    let mut selection = wait(vec![CaseSource::Timer(100)]);
    selection.cases[0].callback = SharedFunction::task_owned(
        FunctionId::new(0),
        "pending".into(),
        vec![Value::Cell(captured)],
        0,
    );
    assert!(!worker.poll_selection(selection).unwrap());
    assert!(weak.upgrade().is_some());
    drop(worker.task_suspension.take());
    assert!(weak.upgrade().is_none());
}

#[test]
fn scheduler_shutdown_releases_parked_pool_selection_captures() {
    use std::sync::Mutex;
    for _ in 0..100 {
        let mut worker = worker();
        worker.task_id = 1;
        let scheduler = Arc::clone(worker.scheduler_ref().unwrap());
        let captured = Arc::new(Mutex::new(Value::Integer(9)));
        let weak = Arc::downgrade(&captured);
        let mut selection = wait(vec![CaseSource::Timer(u64::MAX)]);
        selection.cases[0].callback = SharedFunction::task_owned(
            FunctionId::new(0),
            "pending".into(),
            vec![Value::Cell(captured)],
            1,
        );
        worker.run_selection(selection).unwrap();
        assert!(worker.suspend_requested);
        assert!(
            worker.task_suspension.is_none(),
            "suspension was not transferred"
        );
        assert!(
            weak.upgrade().is_some(),
            "parked selection lost its capture"
        );
        scheduler.request_cancel();
        assert!(worker.callback_continuations.is_empty());
        assert!(
            weak.upgrade().is_none(),
            "cancelled pool wait retained capture"
        );
    }
}

#[test]
fn scheduler_shutdown_releases_normal_selection_cases_without_running_the_callback() {
    use std::sync::Mutex;
    let mut worker = worker();
    let scheduler = Arc::clone(worker.scheduler_ref().unwrap());
    let captured = Arc::new(Mutex::new(Value::Integer(9)));
    let weak = Arc::downgrade(&captured);
    let mut selection = wait(vec![CaseSource::Timer(u64::MAX)]);
    selection.cases[0].callback = SharedFunction::task_owned(
        FunctionId::new(0),
        "pending".into(),
        vec![Value::Cell(captured)],
        0,
    );
    let stopping = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(5));
        scheduler.request_cancel();
    });
    let outcome = worker.run_selection(selection);
    stopping.join().unwrap();
    assert!(outcome.is_err());
    assert!(worker.callback_continuations.is_empty());
    assert!(weak.upgrade().is_none());
}
