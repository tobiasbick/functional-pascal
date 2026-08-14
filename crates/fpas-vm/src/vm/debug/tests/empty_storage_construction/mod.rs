//! Helpers for seeded empty-storage descendant initialization tests.

use fpas_bytecode::VerifiedExecutable;

pub(super) use super::function_value_assignment::{
    assignment_executable as function_assignment_executable, stop_with_functions,
};
pub(super) use super::uninitialized_assignment::{
    assignment_executable, field, named, root, scope_reference, task_assignment_executable,
};
pub(super) use super::*;

pub(super) fn panic_executable() -> VerifiedExecutable {
    super::panic_executable()
}

mod construction;
mod rejection;

const SOURCE: &str =
    include_str!("../../../../../../../tests/debugger/fixtures/empty_storage_construction.fpas");

pub(super) fn construction_executable() -> VerifiedExecutable {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    fpas_compiler::compile(&program).expect("compile empty-storage construction fixture")
}

pub(super) fn make_initial_state() -> DebugExpression {
    DebugExpression::Call {
        callee: Box::new(DebugExpression::Callable("MakeInitialState".to_string())),
        arguments: Vec::new(),
    }
}

pub(super) fn next_call() -> DebugExpression {
    DebugExpression::Call {
        callee: Box::new(DebugExpression::Callable("Next".to_string())),
        arguments: Vec::new(),
    }
}

pub(super) fn index_target(root_name: &str, index: DebugExpression) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: root_name.to_string(),
        selectors: vec![DebugAssignmentSelector::Index(index)],
    }
}

pub(super) fn nested(root_name: &str, fields: &[&str]) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: root_name.to_string(),
        selectors: fields
            .iter()
            .map(|name| DebugAssignmentSelector::Field((*name).to_string()))
            .collect(),
    }
}

pub(super) fn stop_with_empty(session: &mut DebugSession, name: &str) -> u64 {
    for _ in 0..64 {
        if let Ok(stack) = session.stack(0, 1)
            && let Some(frame) = stack.items.first()
            && let Ok(scopes) = session.scopes(frame.id)
            && let Some(locals) = scopes.into_iter().find(|scope| scope.name == "Locals")
            && let Ok(variables) = session.variables(locals.variables_reference, 0, 20)
            && let Some(variable) = variables.items.iter().find(|item| item.name == name)
        {
            if variable.value == "<uninitialized>" {
                return frame.id;
            }
            panic!("{name} already initialized as {}", variable.value);
        }
        let _ = stopped(session.step_into().expect("step toward empty storage"));
    }
    panic!("{name} never became visible uninitialized")
}

pub(super) fn stop_with_initialized(session: &mut DebugSession, name: &str) -> u64 {
    for _ in 0..64 {
        if let Ok(stack) = session.stack(0, 1)
            && let Some(frame) = stack.items.first()
            && let Ok(scopes) = session.scopes(frame.id)
            && let Some(locals) = scopes.into_iter().find(|scope| scope.name == "Locals")
            && let Ok(variables) = session.variables(locals.variables_reference, 0, 20)
            && let Some(variable) = variables.items.iter().find(|item| item.name == name)
            && variable.value != "<uninitialized>"
        {
            return frame.id;
        }
        let _ = stopped(
            session
                .step_into()
                .expect("step toward initialized binding"),
        );
    }
    panic!("{name} never became initialized")
}

pub(super) fn local_value(session: &mut DebugSession, name: &str) -> String {
    let locals = scope_reference(session, "Locals");
    named(
        &session.variables(locals, 0, 20).expect("locals").items,
        name,
    )
    .value
    .clone()
}
