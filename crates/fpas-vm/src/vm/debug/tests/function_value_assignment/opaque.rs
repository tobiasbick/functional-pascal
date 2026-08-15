//! Opaque hosted handles stay non-assignable without a typed host identity contract.

use fpas_bytecode::{Value, VerifiedExecutable};

use super::*;

const SOURCE: &str = r#"program OpaqueIdentityBoundary;

uses Std.Console;

begin
  var Region: SavedRegion := SaveRegion(record
    x := 1;
    y := 1;
    width := 1;
    height := 1;
  end);
  mutable var Copy: SavedRegion := SaveRegion(record
    x := 1;
    y := 2;
    width := 1;
    height := 1;
  end);
  var StopMarker: integer := 0;
end.
"#;

fn compile_opaque() -> VerifiedExecutable {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    fpas_compiler::compile(&program).expect("compile opaque identity fixture")
}

fn run_to_stop(session: &mut DebugSession) -> u64 {
    let line = u32::try_from(
        SOURCE
            .lines()
            .position(|line| line.contains("var StopMarker: integer := 0;"))
            .expect("stop marker")
            .saturating_add(1),
    )
    .expect("line");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line,
            column: None,
        })
        .expect("breakpoint");
    let _ = stopped(session.continue_execution().expect("run to marker"));
    session.stack(0, 1).expect("stack").items[0].id
}

fn opaque_handle(value: &Value) -> u64 {
    match value {
        Value::OpaqueHandle(id) => *id,
        other => panic!("expected an opaque handle, got {}", other.type_name()),
    }
}

#[test]
fn opaque_hosted_assignment_is_rejected_without_copying_raw_ids() {
    let mut session = DebugSession::new(compile_opaque()).expect("debug session");
    let frame = run_to_stop(&mut session);
    let locals = scope_reference(&mut session, "Locals");
    let generation = locals;
    let region = session
        .evaluate_runtime_value(
            &name("Region"),
            Some(frame),
            DebugEvaluationLimits::default(),
        )
        .expect("region");
    let copy = session
        .evaluate_runtime_value(&name("Copy"), Some(frame), DebugEvaluationLimits::default())
        .expect("copy");
    let region_id = opaque_handle(&region);
    let copy_id = opaque_handle(&copy);
    assert_ne!(
        region_id, copy_id,
        "the two SavedRegion values must be distinct one-shot handles"
    );

    let rejected = session
        .set_expression(&root("Copy"), &name("Region"), Some(frame))
        .expect_err("opaque assignment");
    assert_eq!(rejected.kind, DebugErrorKind::VariableValueType);

    let preserved = session
        .variables(generation, 0, 10)
        .expect("generation survives opaque rejection")
        .items;
    assert_eq!(named(&preserved, "Copy").value, "<opaque handle>");
    assert_eq!(named(&preserved, "Region").value, "<opaque handle>");
    let frame = session.stack(0, 1).expect("unchanged frame").items[0].id;
    let after = session
        .evaluate_runtime_value(&name("Copy"), Some(frame), DebugEvaluationLimits::default())
        .expect("copy after rejection");
    assert_eq!(opaque_handle(&after), copy_id);
}
