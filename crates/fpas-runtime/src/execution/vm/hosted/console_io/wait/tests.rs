//! Admission, issue ordering and undelivered native results have independent lifetimes.

use super::*;

fn worker() -> Worker {
    let (program, errors) = fpas_parser::parse("program C; uses Std.Console as C; begin end.");
    assert!(errors.is_empty(), "{errors:?}");
    Worker::new(Arc::new(fpas_compiler::compile(&program).unwrap())).unwrap()
}

fn suspended(worker: &mut Worker) -> Wait {
    let Some(TaskSuspension::ConsoleIo(wait)) = worker.task_suspension.take() else {
        panic!("saved console request")
    };
    *wait
}

#[test]
fn capacity_queues_plain_calls_and_cancelled_tickets_do_not_block_successors() {
    let mut worker = worker();
    let mut permits = (0..LIMIT)
        .map(|_| reserve(worker.hosted.clone()).unwrap())
        .collect::<Vec<_>>();
    let operation = Intrinsic::Console(crate::ConsoleIntrinsic::WriteLn);
    worker
        .start_console_io(operation, &[Value::Str("lost".into())], None)
        .unwrap();
    let cancelled = suspended(&mut worker);
    assert!(matches!(&cancelled.state, State::Queued(_)));
    worker
        .start_console_io(operation, &[Value::Str("kept".into())], None)
        .unwrap();
    let wait = suspended(&mut worker);
    drop(permits.pop());
    assert!(
        !worker.poll_console_io(wait).unwrap(),
        "later output must wait for its predecessor"
    );
    let wait = suspended(&mut worker);
    assert!(matches!(&wait.state, State::Queued(_)));
    assert_eq!(
        worker.hosted.console_jobs.load(Ordering::Acquire),
        LIMIT - 1
    );
    drop(cancelled);
    if !worker.poll_console_io(wait).unwrap() {
        let mut wait = suspended(&mut worker);
        let State::Running(job) = &mut wait.state else {
            panic!("admitted output")
        };
        job.thread.take().unwrap().join().unwrap();
        assert!(worker.poll_console_io(wait).unwrap());
    }
    drop(permits);
    assert_eq!(worker.hosted.console_jobs.load(Ordering::Acquire), 0);
    assert_eq!(
        worker
            .hosted
            .console
            .lock()
            .unwrap()
            .output()
            .lines
            .join("\n"),
        "kept"
    );
}

#[test]
fn abandoned_region_results_are_discarded_without_waiting_on_console_lock() {
    let worker = worker();
    let location = SourceLocation::new(1, 1);
    let call = super::super::request::Call {
        operation: crate::ConsoleIntrinsic::SaveRegion,
        numbers: [0; 4],
        payload: super::super::request::Payload::Rect(crate::stdlib::ConsoleRect {
            x: 1,
            y: 1,
            width: 2,
            height: 1,
        }),
        location,
    };
    let result = call.run(&worker.hosted, || false).unwrap();
    let Output::Region(ref region) = result else {
        panic!("saved region")
    };
    let id = region.id.unwrap();
    let mut console = worker.hosted.console.lock().unwrap();
    drop(result);
    assert_eq!(*worker.hosted.console_regions.lock().unwrap(), [id]);
    super::super::discard_regions(&worker.hosted, &mut console);
    assert!(
        console.discard_region(id, location).is_err(),
        "unpublished region must already be gone"
    );
    assert!(worker.hosted.console_regions.lock().unwrap().is_empty());
}
