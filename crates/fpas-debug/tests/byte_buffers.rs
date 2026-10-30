//! Native bytes retain their complete payload through inspection and detached calls.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "debugger fixture failures stay local"
)]

use fpas_runtime::execution::{
    DebugAssignmentTarget, DebugExpression, DebugRunResult, DebugSession, SourceBreakpoint,
};

fn name(value: &str) -> DebugExpression {
    DebugExpression::Name(value.into())
}

#[test]
fn byte_inspection_detached_mutation_and_event_capture_preserve_live_bytes() {
    let source = "program BufferDebug;
uses Std.Bytes as Bytes;
function Change(var Data: Bytes.ByteBuffer): integer;
begin
  discard Data.SetAtInPlace(39, 99);
  return Data[39];
end function;
begin
  var Data := Bytes.Create(40).Unwrap();
  discard Data.SetAtInPlace(39, 255);
  discard Data.ReserveInPlace(128);
  var Marker := 0;
  Marker := Marker + Data[39];
end.";
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let image = fpas_compiler::compile(&program).expect("compile byte debugger fixture");
    let mut session = DebugSession::new(image).expect("session");
    session.start_recording();
    let breakpoint = session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".into(),
            line: 13,
            column: None,
        })
        .expect("breakpoint");
    assert!(breakpoint.is_verified(), "{breakpoint:?}");
    assert!(matches!(
        session.continue_execution().expect("stop"),
        DebugRunResult::Stopped(_)
    ));
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let summary = session
        .evaluate(&name("Data"), Some(frame))
        .expect("buffer inspection");
    assert_eq!(summary.type_name, "Std.Bytes.ByteBuffer");
    assert!(summary.value.contains("Length := 40"));
    assert!(summary.value.ends_with("...')"));
    let children = session
        .variables(summary.variables_reference, 0, 10)
        .expect("buffer children");
    assert!(
        children
            .items
            .iter()
            .any(|value| value.name == "Length" && value.value == "40")
    );
    assert!(
        children
            .items
            .iter()
            .any(|value| value.name == "Capacity" && value.value == "128")
    );
    let last = DebugExpression::Index {
        base: Box::new(name("Data")),
        index: Box::new(DebugExpression::Integer(39)),
    };
    assert_eq!(
        session
            .evaluate(&last, Some(frame))
            .expect("byte beyond preview")
            .value,
        "255"
    );
    let before = session.last_stop().clone();
    let result = session
        .evaluate(
            &DebugExpression::Call {
                callee: Box::new(DebugExpression::Callable("Change".into())),
                arguments: vec![DebugExpression::VarArgument(DebugAssignmentTarget {
                    root: "Data".into(),
                    selectors: Vec::new(),
                })],
            },
            Some(frame),
        )
        .expect("detached buffer mutation");
    assert_eq!(result.value, "99");
    assert_eq!(
        session
            .evaluate(&last, Some(frame))
            .expect("unchanged live byte")
            .value,
        "255"
    );
    assert_eq!(session.last_stop(), &before);
    assert!(matches!(
        session
            .continue_execution()
            .expect("complete under recording"),
        DebugRunResult::Terminated(_)
    ));
    assert!(session.is_recording());
    assert!(!session.recording_events().is_empty());
}
