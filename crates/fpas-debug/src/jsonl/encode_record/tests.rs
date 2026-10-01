//! Runtime event locations must not manufacture source coordinates.

use super::*;
use fpas_diagnostics::{Diagnostic, codes::RUNTIME_PROGRAM_PANIC};

#[test]
fn runtime_event_without_source_preserves_null_coordinates() {
    let encoded = encode_record(DebugRecord::Event(DebugEvent::RuntimeError {
        diagnostic: Diagnostic::error_without_source(RUNTIME_PROGRAM_PANIC, "panic: failure", None),
        task_id: 0,
    }));
    assert_eq!(encoded["event"], "runtime_error");
    assert_eq!(encoded["body"]["line"], Value::Null);
    assert_eq!(encoded["body"]["column"], Value::Null);
    assert_eq!(encoded["body"]["source_id"], Value::Null);
    assert_eq!(encoded["body"]["code"], "F4010");
}
