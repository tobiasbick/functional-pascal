//! Compiled cell-capturing named-routine assignment coverage.

use fpas_bytecode::{SharedFunction, Value, VerifiedExecutable};

pub(super) use super::*;

const SOURCE: &str = include_str!(
    "../../../../../../../tests/debugger/fixtures/cell_capturing_routine_assignment.fpas"
);

mod rejection;
mod success;

pub(super) fn compile_fixture() -> VerifiedExecutable {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    fpas_compiler::compile(&program).expect("compile cell-capturing-routine fixture")
}

pub(super) fn line(needle: &str) -> u32 {
    u32::try_from(
        SOURCE
            .lines()
            .position(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("missing marker {needle}"))
            .saturating_add(1),
    )
    .expect("line")
}

pub(super) fn run_to(session: &mut DebugSession, needle: &str) -> u64 {
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: line(needle),
            column: None,
        })
        .expect("breakpoint");
    let _ = stopped(session.continue_execution().expect("run to marker"));
    session.stack(0, 8).expect("stack").items[0].id
}

pub(super) fn run_to_hit(session: &mut DebugSession, needle: &str, hits: usize) -> u64 {
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: line(needle),
            column: None,
        })
        .expect("breakpoint");
    for _ in 0..hits {
        let _ = stopped(session.continue_execution().expect("run to marker hit"));
    }
    session.stack(0, 8).expect("stack").items[0].id
}

pub(super) fn root(name: &str) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: name.to_string(),
        selectors: Vec::new(),
    }
}

pub(super) fn field(root_name: &str, name: &str) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: root_name.to_string(),
        selectors: vec![DebugAssignmentSelector::Field(name.to_string())],
    }
}

pub(super) fn name(name: &str) -> DebugExpression {
    DebugExpression::Name(name.to_string())
}

pub(super) fn call(callee: &str, value: i64) -> DebugExpression {
    DebugExpression::Call {
        callee: Box::new(name(callee)),
        arguments: vec![DebugExpression::Integer(value)],
    }
}

pub(super) fn rendered(
    session: &mut DebugSession,
    expression: DebugExpression,
    frame: u64,
) -> String {
    session
        .evaluate(&expression, Some(frame))
        .expect("debug evaluation must succeed")
        .value
}

pub(super) fn runtime(session: &DebugSession, expression: &DebugExpression, frame: u64) -> Value {
    session
        .evaluate_runtime_value(expression, Some(frame), DebugEvaluationLimits::default())
        .expect("runtime value")
}

pub(super) fn as_function(value: &Value) -> &SharedFunction {
    match value {
        Value::Function(function) => function,
        other => panic!("expected function, got {}", other.type_name()),
    }
}

pub(super) fn cell_arc(value: &Value) -> &std::sync::Arc<std::sync::Mutex<Value>> {
    match value {
        Value::Cell(cell) => cell,
        other => panic!("expected cell, got {}", other.type_name()),
    }
}

pub(super) fn scope_reference(session: &mut DebugSession, scope_name: &str) -> u64 {
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .scopes(frame)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == scope_name)
        .expect("requested scope")
        .variables_reference
}

pub(super) fn named<'a>(items: &'a [crate::DebugVariable], name: &str) -> &'a crate::DebugVariable {
    items
        .iter()
        .find(|item| item.name == name)
        .unwrap_or_else(|| panic!("missing {name}"))
}
