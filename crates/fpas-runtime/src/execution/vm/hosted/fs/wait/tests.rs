//! Native ownership, cancellation behind alias locks, and admission before I/O.

use super::*;
use crate::execution::vm::hosted::fs::tests::Fixture;
use crate::stdlib::Console;
use std::time::Duration;

fn worker() -> Worker {
    let (program, errors) = fpas_parser::parse("program F; uses Std.Io.Fs as Fs; begin end.");
    assert!(errors.is_empty(), "{errors:?}");
    Worker::new(Arc::new(fpas_compiler::compile(&program).unwrap())).unwrap()
}

fn call(worker: &Worker, op: FsIntrinsic, args: &[Value]) -> Call {
    Call::prepare(worker, op, args).unwrap().unwrap()
}

#[test]
fn file_job_admission_precedes_truncation_and_counts_undelivered_opens() {
    let worker = worker();
    let fixture = Fixture::new();
    let path = fixture.path("admission");
    std::fs::write(&path, b"kept").unwrap();
    let mut permits = (0..MAX_JOBS)
        .map(|_| Permit::acquire(worker.hosted.clone()).unwrap())
        .collect::<Vec<_>>();
    let args = [Value::Str(path.clone().into()), Value::Integer(1)];
    assert!(matches!(
        Job::start(
            worker.hosted.clone(),
            call(&worker, FsIntrinsic::Open, &args)
        ),
        Err(Failure::Limit(64))
    ));
    assert_eq!(std::fs::read(&path).unwrap(), b"kept");
    drop(permits.pop());
    let mut job = Job::start(
        worker.hosted.clone(),
        call(&worker, FsIntrinsic::Open, &args),
    )
    .unwrap();
    job.thread.take().unwrap().join().unwrap();
    assert_eq!(worker.hosted.file_jobs.load(Ordering::Acquire), MAX_JOBS);
    let handles = worker.hosted.files.handles();
    assert_eq!(handles.len(), 1);
    let entry = worker
        .hosted
        .files
        .entry(handles[0], &mut || false)
        .unwrap();
    assert!(entry.state.lock().unwrap().is_some());
    drop(job);
    assert!(worker.hosted.files.handles().is_empty());
    assert!(entry.state.lock().unwrap().is_none());
    drop(permits);
    assert_eq!(worker.hosted.file_jobs.load(Ordering::Acquire), 0);
}

#[test]
fn abandoning_jobs_behind_a_busy_alias_never_executes_queued_io() {
    let worker = worker();
    let fixture = Fixture::new();
    let path = fixture.path("queued");
    std::fs::write(&path, b"kept\n").unwrap();
    let handle = worker.hosted.files.open(&path, 3, || false).unwrap();
    let entry = worker.hosted.files.entry(handle, &mut || false).unwrap();
    for (op, extra) in [
        (FsIntrinsic::Read, vec![Value::Integer(1)]),
        (FsIntrinsic::ReadLine, vec![]),
        (
            FsIntrinsic::Write,
            vec![Value::Array(vec![Value::Integer(0)].into())],
        ),
        (FsIntrinsic::WriteString, vec![Value::Str("lost".into())]),
        (FsIntrinsic::WriteLine, vec![Value::Str("lost".into())]),
        (
            FsIntrinsic::Seek,
            vec![Value::Integer(2), Value::Integer(0)],
        ),
        (FsIntrinsic::Tell, vec![]),
        (FsIntrinsic::Size, vec![]),
        (FsIntrinsic::SetLength, vec![Value::Integer(0)]),
        (FsIntrinsic::Flush, vec![]),
        (FsIntrinsic::Close, vec![]),
    ] {
        let mut args = vec![Value::OpaqueHandle(handle)];
        args.extend(extra);
        let held = entry.state.lock().unwrap();
        let job = Job::start(worker.hosted.clone(), call(&worker, op, &args)).unwrap();
        let (sent, received) = mpsc::channel();
        let dropped = std::thread::spawn(move || {
            drop(job);
            sent.send(()).unwrap();
        });
        let result = received.recv_timeout(Duration::from_secs(5));
        drop(held);
        dropped.join().unwrap();
        result.expect("a queued operation must cancel while the alias remains locked");
        assert_eq!(worker.hosted.file_jobs.load(Ordering::Acquire), 0);
        assert_eq!(
            worker.hosted.files.tell(handle, || false).unwrap(),
            0,
            "{op:?}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"kept\n", "{op:?}");
    }
}

#[test]
fn shutdown_signal_does_not_wait_for_a_busy_file() {
    let hosted = Arc::new(HostedState::new(Console::new(), Vec::new()));
    let fixture = Fixture::new();
    let handle = hosted
        .files
        .open(&fixture.path("shutdown"), 1, || false)
        .unwrap();
    let entry = hosted.files.entry(handle, &mut || false).unwrap();
    let held = entry.state.lock().unwrap();
    hosted.shutdown_blocking_operations();
    assert!(hosted.files.is_stopping());
    drop(held);
    hosted.close_file_handles();
    assert!(entry.state.lock().unwrap().is_none());
}

#[test]
fn token_cancellation_finishes_behind_a_busy_alias_without_advancing_it() {
    let worker = worker();
    let fixture = Fixture::new();
    let path = fixture.path("token");
    std::fs::write(&path, b"kept\n").unwrap();
    let handle = worker.hosted.files.open(&path, 3, || false).unwrap();
    let entry = worker.hosted.files.entry(handle, &mut || false).unwrap();
    for (op, extra) in [
        (FsIntrinsic::ReadWithCancellation, vec![Value::Integer(1)]),
        (FsIntrinsic::ReadLineWithCancellation, vec![]),
        (
            FsIntrinsic::WriteWithCancellation,
            vec![Value::Array(vec![Value::Integer(0)].into())],
        ),
        (
            FsIntrinsic::WriteStringWithCancellation,
            vec![Value::Str("lost".into())],
        ),
        (
            FsIntrinsic::WriteLineWithCancellation,
            vec![Value::Str("lost".into())],
        ),
    ] {
        let source = worker.hosted.cancellations.create_source();
        let token = worker.hosted.cancellations.token(source).unwrap();
        let mut args = vec![Value::OpaqueHandle(handle)];
        args.extend(extra);
        args.push(Value::OpaqueHandle(token));
        let held = entry.state.lock().unwrap();
        let mut job = Job::start(worker.hosted.clone(), call(&worker, op, &args)).unwrap();
        worker.hosted.cancellations.cancel(source).unwrap();
        let (sent, received) = mpsc::channel();
        let joining = std::thread::spawn(move || {
            job.thread.take().unwrap().join().unwrap();
            sent.send(()).unwrap();
            job
        });
        let outcome = received.recv_timeout(Duration::from_secs(5));
        drop(held);
        let job = joining.join().unwrap();
        outcome.expect("token cancellation must not wait for the alias lock");
        assert!(matches!(job.poll(), Some(Err(Failure::Cancelled))));
        drop(job);
        assert_eq!(worker.hosted.files.tell(handle, || false).unwrap(), 0);
        assert_eq!(std::fs::read(&path).unwrap(), b"kept\n");
    }
}

#[test]
fn plain_queries_queue_at_capacity_without_changing_their_return_type() {
    let mut worker = worker();
    let fixture = Fixture::new();
    let path = fixture.path("exists");
    std::fs::write(&path, b"kept").unwrap();
    let arguments = [Value::Str(path.into())];
    let mut permits = (0..MAX_JOBS)
        .map(|_| Permit::acquire(worker.hosted.clone()).unwrap())
        .collect::<Vec<_>>();
    assert!(
        worker
            .start_fs_wait(FsIntrinsic::Exists, &arguments, None)
            .unwrap()
            .is_none()
    );
    let Some(TaskSuspension::Fs(wait)) = worker.task_suspension.take() else {
        panic!("saved filesystem query")
    };
    assert!(matches!(&wait.state, State::Queued(_)));
    drop(permits.pop());
    if !worker.poll_fs_wait(*wait).unwrap() {
        let Some(TaskSuspension::Fs(mut wait)) = worker.task_suspension.take() else {
            panic!("running query")
        };
        let State::Running(job) = &mut wait.state else {
            panic!("admitted query")
        };
        if let Some(thread) = job.thread.take() {
            thread.join().unwrap();
        }
        assert!(worker.poll_fs_wait(*wait).unwrap());
    }
    let outcome = call(&worker, FsIntrinsic::Exists, &arguments).run(&worker.hosted, || false);
    let value = worker
        .fs_result(FsIntrinsic::Exists, outcome, SourceLocation::new(1, 1))
        .unwrap();
    assert_eq!(value, Value::Boolean(true));
    drop(permits);
    assert_eq!(worker.hosted.file_jobs.load(Ordering::Acquire), 0);
}

#[test]
fn path_queries_do_not_depend_on_the_file_handle_registry() {
    let worker = worker();
    let fixture = Fixture::new();
    let path = fixture.path("independent");
    std::fs::write(&path, b"kept").unwrap();
    worker.hosted.close_file_handles();
    let mut job = Job::start(
        worker.hosted.clone(),
        call(&worker, FsIntrinsic::IsFile, &[Value::Str(path.into())]),
    )
    .unwrap();
    job.thread.take().unwrap().join().unwrap();
    let value = worker
        .fs_result(
            FsIntrinsic::IsFile,
            job.poll().unwrap(),
            SourceLocation::new(1, 1),
        )
        .unwrap();
    assert_eq!(value, Value::Boolean(true));
}

#[test]
fn native_callback_can_return_before_file_join_and_late_open_is_closed() {
    let worker = worker();
    let fixture = Fixture::new();
    let handle = worker
        .hosted
        .files
        .open(&fixture.path("retired"), 1, || false)
        .unwrap();
    let (release, wait) = mpsc::channel();
    let (sender, receiver) = mpsc::channel();
    let job = Job {
        permit: Some(Permit::acquire(worker.hosted.clone()).unwrap()),
        stop: Arc::new(AtomicBool::new(false)),
        receiver: Mutex::new(receiver),
        thread: Some(std::thread::spawn(move || {
            wait.recv_timeout(Duration::from_secs(10))
                .expect("host callback must return before joining file I/O");
            sender.send(Ok(Output::Handle(handle))).unwrap();
        })),
    };
    super::super::super::native::defer_joins(|| drop(job));
    assert_eq!(worker.hosted.file_jobs.load(Ordering::Acquire), 1);
    release.send(()).unwrap();
    super::super::super::native::join_retired(&worker.hosted);
    assert_eq!(worker.hosted.file_jobs.load(Ordering::Acquire), 0);
    assert!(worker.hosted.files.handles().is_empty());
}
