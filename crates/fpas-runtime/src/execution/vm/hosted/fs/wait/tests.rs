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
