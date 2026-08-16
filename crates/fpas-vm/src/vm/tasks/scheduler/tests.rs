//! Exact retained-failure transition tests.

use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

use super::TaskScheduler;
use crate::vm::{TaskResultPoll, runtime_error};

fn failure(message: &str) -> crate::vm::VmError {
    runtime_error(
        RUNTIME_PROGRAM_PANIC,
        message,
        "Recover explicitly in the debugger.",
        SourceLocation::new(1, 1),
    )
}

#[test]
fn retained_failure_transitions_require_the_exact_diagnostic() {
    let scheduler = TaskScheduler::new();
    scheduler.register_result(1);
    let original = failure("original");
    scheduler.store_failure(1, original.clone());

    assert!(!scheduler.recover_failure(1, &failure("different")));
    assert!(matches!(scheduler.poll_result(1), TaskResultPoll::Failed(error) if error == original));
    assert!(scheduler.recover_failure(1, &original));
    assert!(matches!(scheduler.poll_result(1), TaskResultPoll::Pending));

    scheduler.store_failure(1, original.clone());
    assert!(!scheduler.replace_failure(1, &failure("different"), Value::Integer(9)));
    assert!(matches!(scheduler.poll_result(1), TaskResultPoll::Failed(error) if error == original));
    assert!(scheduler.replace_failure(1, &original, Value::Integer(9)));
    assert!(matches!(
        scheduler.poll_result(1),
        TaskResultPoll::Available(Value::Integer(9))
    ));
    assert!(matches!(scheduler.poll_result(1), TaskResultPoll::Consumed));
}
