//! Session helpers for debugger task-handle assignment tests.

use super::super::*;

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

pub(super) fn stop_with_tasks(session: &mut DebugSession) {
    for _ in 0..48 {
        if let Ok(locals) = session
            .stack(0, 1)
            .and_then(|stack| session.scopes(stack.items[0].id))
            && let Some(locals) = locals.into_iter().find(|scope| scope.name == "Locals")
            && let Ok(variables) = session.variables(locals.variables_reference, 0, 40)
            && named(&variables.items, "Current")
                .value
                .starts_with("<task")
            && named(&variables.items, "StopMarker").value != "<uninitialized>"
        {
            return;
        }
        let _ = stopped(
            session
                .step_into()
                .expect("step toward initialized task handles"),
        );
    }
    panic!("task handles never became initialized")
}
