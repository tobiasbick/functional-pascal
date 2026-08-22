//! Compiled capturing-routine assignment coverage.

use fpas_bytecode::VerifiedExecutable;

pub(super) use super::*;

const SOURCE: &str =
    include_str!("../../../../../../../tests/debugger/fixtures/capturing_routine_assignment.fpas");

mod rejection;
mod success;

pub(super) fn compile_fixture() -> VerifiedExecutable {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    fpas_compiler::compile(&program).expect("compile capturing-routine fixture")
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
