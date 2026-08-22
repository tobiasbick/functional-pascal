//! Engine-interface tests that never construct JSON arguments.

#![allow(
    clippy::expect_used,
    reason = "engine tests use expect to keep fixture failures local"
)]

use super::{DebugEngine, DebugOp, DebugRecord, DebugRequest, DebugStatus, EngineFailure};
use crate::PreparedDebugTarget;

const SOURCE: &str = r#"program EngineSurface;

begin
  mutable var Value: integer := 40;
  Value := Value + 2
end.
"#;

fn engine() -> DebugEngine {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile engine fixture");
    DebugEngine::new(PreparedDebugTarget::new(executable, Vec::new())).expect("debug engine")
}

fn failure_code(records: &[DebugRecord]) -> &str {
    let failure_message = format!("expected engine failure response, got {records:?}");
    records
        .first()
        .and_then(|record| match record {
            DebugRecord::Response {
                outcome: Err(EngineFailure { code, .. }),
                ..
            } => Some(code.as_str()),
            _ => None,
        })
        .expect(&failure_message)
}

#[test]
fn evaluate_before_initialize_is_invalid_state() {
    let mut engine = engine();
    let records = engine.execute(DebugRequest::new(
        1,
        DebugOp::Evaluate {
            expression: "Value".to_string(),
            frame_id: None,
            async_eval: false,
        },
    ));
    assert_eq!(failure_code(&records), "invalid_state");
    assert_eq!(engine.status(), DebugStatus::Created);
}

#[test]
fn initialize_then_launch_stops_on_entry_without_json() {
    let mut engine = engine();
    let initialized = engine.execute(DebugRequest::new(1, DebugOp::Initialize));
    assert!(
        initialized
            .iter()
            .any(|record| matches!(record, DebugRecord::Event(super::DebugEvent::Initialized)))
    );
    assert_eq!(engine.status(), DebugStatus::Initialized);

    let launched = engine.execute(DebugRequest::new(
        2,
        DebugOp::Launch {
            stop_on_entry: true,
        },
    ));
    assert!(
        launched
            .iter()
            .any(|record| matches!(record, DebugRecord::Event(super::DebugEvent::Stopped(_))))
    );
    assert_eq!(engine.status(), DebugStatus::Stopped);
}

#[test]
fn duplicate_request_id_is_rejected_at_the_engine() {
    let mut engine = engine();
    let _ = engine.execute(DebugRequest::new(1, DebugOp::Initialize));
    let records = engine.execute(DebugRequest::new(
        1,
        DebugOp::Launch {
            stop_on_entry: true,
        },
    ));
    assert_eq!(failure_code(&records), "invalid_request");
}
