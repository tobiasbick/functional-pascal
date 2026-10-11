//! Blocking main-task waits remain cancellable when their detached target stays pending.

use std::sync::{Arc, Barrier, mpsc};
use std::time::Duration;

use fpas_bytecode::{Intrinsic, TaskIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

use super::worker;
use crate::vm::TaskResultPoll;

#[test]
fn unrelated_child_failure_interrupts_blocking_wait_and_wait_all() {
    for operation in [
        TaskIntrinsic::Wait,
        TaskIntrinsic::WaitAll,
        TaskIntrinsic::WaitAny,
    ] {
        let mut worker = worker();
        worker.enter_task_scope().expect("scope");
        let scheduler = Arc::clone(worker.scheduler.as_ref().expect("scheduler"));
        let scope = worker.spawning_scope().expect("scope identity");
        scheduler.scopes.enroll(scope, 1).expect("owned child");
        scheduler.register_result(1);
        scheduler.register_result(2);
        let failure =
            worker.runtime_error(RUNTIME_PROGRAM_PANIC, "owned child failed", "scope failure");
        let arguments = match operation {
            TaskIntrinsic::Wait => vec![Value::Task(2)],
            _ => vec![Value::Array(vec![Value::Task(2)].into())],
        };
        let started = Arc::new(Barrier::new(2));
        let thread_started = Arc::clone(&started);
        let (send, receive) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            thread_started.wait();
            let error = worker
                .task_intrinsic(Intrinsic::Task(operation), &arguments, None)
                .expect_err("scope cancellation must interrupt the wait");
            send.send(worker.unwind_task_scopes_blocking(error))
                .expect("outcome");
        });
        started.wait();
        // Let the root enter its blocking wait before publishing an unrelated failure.
        std::thread::sleep(Duration::from_millis(20));
        scheduler.store_failure(1, failure);
        let outcome = receive.recv_timeout(Duration::from_secs(1));
        if outcome.is_err() {
            scheduler.request_cancel();
        }
        thread.join().expect("join waiter");
        let error = outcome.expect("child failure must wake a root waiting on another task");
        assert_eq!(error.code, RUNTIME_PROGRAM_PANIC, "{error:?}");
        assert_eq!(error.message, "owned child failed");
        assert!(
            matches!(scheduler.poll_result(2), TaskResultPoll::Pending),
            "cancelling a scope must not complete its detached wait target"
        );
    }
}
