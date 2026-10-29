//! Native admission precedes effects; undelivered resources remain owned by their job.

use super::*;
use crate::execution::vm::hosted::proc::{AcpLaunch, AcpTerminals, PipedProcesses};
use fpas_acp::connection::Limits;

fn worker() -> Worker {
    let (program, errors) = fpas_parser::parse(
        "program A; uses Std.Ai.Acp as A; begin const M := A.ConnectMessages(A.DefaultOptions()).Unwrap(); end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    Worker::new(Arc::new(fpas_compiler::compile(&program).unwrap())).unwrap()
}
fn call(worker: &Worker) -> Call {
    let options = worker
        .execute_acp_connection(Op::DefaultOptions, &[], SourceLocation::new(1, 1))
        .unwrap();
    Call::prepare(worker, Op::ConnectMessages, &[options]).unwrap()
}

#[test]
fn four_argument_helpers_reject_truncated_calls_before_responder_lookup() {
    let worker = worker();
    for op in [
        Op::HostWaitForTerminalExit,
        Op::HostRunTerminalAuthentication,
        Op::HostV2RunTerminalAuthentication,
    ] {
        assert!(matches!(
            Call::prepare(&worker, op, &[Value::OpaqueHandle(1), Value::OptionNone]),
            Err(Fault::InvalidArgument("Arguments"))
        ));
    }
}

#[test]
fn capacity_rejects_before_connection_creation_and_undelivered_results_close() {
    let worker = worker();
    let mut permits = (0..LIMIT)
        .map(|_| Permit::reserve(worker.hosted.clone()).unwrap())
        .collect::<Vec<_>>();
    assert!(matches!(
        Job::start(worker.hosted.clone(), call(&worker)),
        Err(Fault::LimitExceeded("NativeJobs", LIMIT))
    ));
    drop(permits.pop());
    let mut job = Job::start(worker.hosted.clone(), call(&worker)).unwrap();
    job.thread.take().unwrap().join().unwrap();
    assert_eq!(worker.hosted.acp_jobs.load(Ordering::Acquire), LIMIT);
    let output = job.poll().unwrap().unwrap();
    let Output::Connection(ref owned) = output else {
        panic!("connected result")
    };
    let id = owned.id.unwrap();
    assert!(worker.hosted.acp.connection(id).is_ok());
    drop(output);
    assert!(worker.hosted.acp.connection(id).is_err());
    drop(job);
    drop(permits);
    assert_eq!(worker.hosted.acp_jobs.load(Ordering::Acquire), 0);
}

#[test]
fn native_callback_defers_join_and_cleanup_of_an_unpublished_connection() {
    let worker = worker();
    let mut job = Job::start(worker.hosted.clone(), call(&worker)).unwrap();
    job.thread.take().unwrap().join().unwrap();
    native::defer_joins(|| drop(job));
    assert_eq!(worker.hosted.acp_jobs.load(Ordering::Acquire), 1);
    native::join_retired(&worker.hosted);
    assert_eq!(worker.hosted.acp_jobs.load(Ordering::Acquire), 0);
}

#[test]
fn unpublished_terminal_result_releases_its_process_and_slot() {
    let processes = Arc::new(PipedProcesses::default());
    let owner = AcpTerminals::new(processes, Limits::default());
    let mut reserved = owner.reserve("session", None).unwrap();
    owner
        .start_controlled(
            &mut reserved,
            &AcpLaunch {
                command: std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                arguments: vec!["--list".into()],
                directory: None,
                environment: Vec::new(),
            },
            || false,
        )
        .unwrap();
    let id = reserved.id.clone();
    drop(reserved);
    assert!(owner.validate_id(&id, "session", || false).is_ok());
    drop(super::super::output::Terminal {
        id: Some(id.clone()),
        owner: owner.clone(),
    });
    assert!(!owner.active());
    assert!(owner.validate_id(&id, "session", || false).is_err());
}
