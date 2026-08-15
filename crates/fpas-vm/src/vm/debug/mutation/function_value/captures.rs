//! Bounded non-executing eligibility checks for retained function capture graphs.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::collections::HashSet;

use fpas_bytecode::{SharedFunction, Value};

use super::super::super::types::{DebugErrorKind, DebugSessionError};

/// Prove a function value is not task-bound and its capture graph is assignable.
pub(super) fn require_eligible(
    function: &SharedFunction,
    max_depth: usize,
    max_values: usize,
) -> Result<(), DebugSessionError> {
    if function.task_bound {
        return Err(ownership(
            "source function is task-bound",
            "Assign a non-task-bound function whose captures contain no cells, tasks, or opaque handles.",
        ));
    }
    let mut state = WalkState {
        visited: HashSet::new(),
        values: 0,
        max_depth,
        max_values,
    };
    state.visited.insert(identity_of_function(function));
    function
        .captures
        .iter()
        .try_for_each(|capture| walk(capture, 1, &mut state))?;
    Ok(())
}

/// Prove a task-owned function's capture graph without locking cell payloads.
pub(super) fn require_task_owned(
    function: &SharedFunction,
    max_depth: usize,
    max_values: usize,
) -> Result<(), DebugSessionError> {
    if !function.task_bound || function.owner_task.is_none() {
        return Err(ownership(
            "constructed function is missing a runtime task owner",
            "Assign a named nested routine from the selected live owner task so mutable captures stay on that task.",
        ));
    }
    let mut state = WalkState {
        visited: HashSet::new(),
        values: 0,
        max_depth,
        max_values,
    };
    state.visited.insert(identity_of_function(function));
    for capture in &function.captures {
        match capture {
            Value::Cell(cell) => {
                if !state.visited.insert(std::sync::Arc::as_ptr(cell) as usize) {
                    continue;
                }
                state.values = state.values.saturating_add(1);
                if state.values > state.max_values {
                    return Err(limit(
                        format!(
                            "function capture graph exceeds detached-value limit {}",
                            state.max_values
                        ),
                        "Use a function whose retained capture graph is smaller than the evaluation value limit.",
                    ));
                }
            }
            other => walk(other, 1, &mut state)?,
        }
    }
    Ok(())
}

struct WalkState {
    visited: HashSet<usize>,
    values: usize,
    max_depth: usize,
    max_values: usize,
}

fn walk(value: &Value, depth: usize, state: &mut WalkState) -> Result<(), DebugSessionError> {
    if depth > state.max_depth {
        return Err(limit(
            format!(
                "function capture graph exceeds depth limit {}",
                state.max_depth
            ),
            "Use a function whose retained captures are shallower than the evaluation depth limit.",
        ));
    }
    if let Some(identity) = shared_identity(value)
        && !state.visited.insert(identity)
    {
        return Ok(());
    }
    state.values = state.values.saturating_add(1);
    if state.values > state.max_values {
        return Err(limit(
            format!(
                "function capture graph exceeds detached-value limit {}",
                state.max_values
            ),
            "Use a function whose retained capture graph is smaller than the evaluation value limit.",
        ));
    }
    match value {
        Value::Cell(_) => Err(ownership(
            "source function captures a mutable cell",
            "Assign a non-task-bound function whose captures contain no cells, tasks, or opaque handles.",
        )),
        Value::Task(_) => Err(ownership(
            "source function captures a task handle",
            "Assign a non-task-bound function whose captures contain no cells, tasks, or opaque handles.",
        )),
        Value::OpaqueHandle(_) => Err(ownership(
            "source function captures an opaque handle",
            "Assign a non-task-bound function whose captures contain no cells, tasks, or opaque handles.",
        )),
        Value::Function(function) if function.task_bound => Err(ownership(
            "source function captures a nested task-bound function",
            "Assign a non-task-bound function whose captures contain no cells, tasks, or opaque handles.",
        )),
        Value::Function(function) => function
            .captures
            .iter()
            .try_for_each(|capture| walk(capture, depth.saturating_add(1), state)),
        Value::Array(values) => values
            .iter()
            .try_for_each(|value| walk(value, depth.saturating_add(1), state)),
        Value::Dict(entries) => {
            for (key, value) in entries.iter() {
                walk(key, depth.saturating_add(1), state)?;
                walk(value, depth.saturating_add(1), state)?;
            }
            Ok(())
        }
        Value::Record(record) => record
            .body()
            .values
            .iter()
            .try_for_each(|value| walk(value, depth.saturating_add(1), state)),
        Value::Enum(enumeration) => enumeration
            .body()
            .values
            .iter()
            .try_for_each(|value| walk(value, depth.saturating_add(1), state)),
        Value::ResultOk(inner) | Value::ResultError(inner) | Value::OptionSome(inner) => {
            walk(inner, depth.saturating_add(1), state)
        }
        Value::Integer(_)
        | Value::Real(_)
        | Value::Boolean(_)
        | Value::Str(_)
        | Value::Unit
        | Value::OptionNone => Ok(()),
    }
}

fn shared_identity(value: &Value) -> Option<usize> {
    Some(match value {
        Value::Array(values) => std::ptr::from_ref(&**values) as usize,
        Value::Dict(entries) => std::ptr::from_ref(&**entries) as usize,
        Value::Record(record) => std::ptr::from_ref(record.body()) as usize,
        Value::Enum(enumeration) => std::ptr::from_ref(enumeration.body()) as usize,
        Value::Function(function) => identity_of_function(function),
        Value::Cell(cell) => std::sync::Arc::as_ptr(cell) as usize,
        _ => return None,
    })
}

fn identity_of_function(function: &SharedFunction) -> usize {
    std::ptr::from_ref(&**function) as usize
}

fn ownership(detail: &str, hint: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!("debug function assignment is rejected: {detail}"),
        hint: hint.to_string(),
    }
}

fn limit(message: String, hint: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::EvaluationLimit,
        message,
        hint: hint.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fpas_bytecode::FunctionId;
    use std::sync::{Arc, Mutex};

    fn function(captures: Vec<Value>, task_bound: bool) -> SharedFunction {
        let value = if task_bound {
            Value::task_owned_function(FunctionId::new(1), "work".to_string(), captures, 1)
        } else {
            Value::function(FunctionId::new(1), "work".to_string(), captures)
        };
        match value {
            Value::Function(function) => function,
            other => panic!("expected function, got {}", other.type_name()),
        }
    }

    #[test]
    fn immutable_aggregates_and_nested_safe_functions_are_eligible() {
        let nested = Value::function(
            FunctionId::new(2),
            "nested".to_string(),
            vec![Value::Integer(1)],
        );
        let captured = function(
            vec![
                Value::Integer(1),
                Value::Array(vec![Value::Boolean(true)].into()),
                Value::dict(vec![(
                    Value::Str("k".into()),
                    Value::OptionSome(Box::new(Value::Integer(2))),
                )]),
                nested,
            ],
            false,
        );
        require_eligible(&captured, 8, 64).expect("safe graph");
    }

    #[test]
    fn task_bound_functions_and_forbidden_captures_are_rejected() {
        let bound = function(Vec::new(), true);
        let error = require_eligible(&bound, 8, 64).expect_err("task-bound");
        assert_eq!(error.kind, DebugErrorKind::VariableValueType);
        assert!(error.message.contains("task-bound"), "{error:?}");
        assert!(error.hint.contains("non-task-bound"), "{}", error.hint);

        let cell = function(
            vec![Value::Cell(Arc::new(Mutex::new(Value::Integer(1))))],
            false,
        );
        assert!(
            require_eligible(&cell, 8, 64)
                .expect_err("cell")
                .message
                .contains("cell")
        );
        let task = function(vec![Value::Task(3)], false);
        assert!(
            require_eligible(&task, 8, 64)
                .expect_err("task")
                .message
                .contains("task handle")
        );
        let opaque = function(vec![Value::OpaqueHandle(4)], false);
        assert!(
            require_eligible(&opaque, 8, 64)
                .expect_err("opaque")
                .message
                .contains("opaque")
        );
        let nested_bound = function(
            vec![Value::task_owned_function(
                FunctionId::new(2),
                "inner".to_string(),
                Vec::new(),
                1,
            )],
            false,
        );
        assert!(
            require_eligible(&nested_bound, 8, 64)
                .expect_err("nested")
                .message
                .contains("nested task-bound")
        );
    }

    #[test]
    fn nested_cell_inside_array_is_rejected_without_executing() {
        let nested = function(
            vec![Value::Array(
                vec![Value::Cell(Arc::new(Mutex::new(Value::Integer(1))))].into(),
            )],
            false,
        );
        let error = require_eligible(&nested, 8, 64).expect_err("nested cell");
        assert_eq!(error.kind, DebugErrorKind::VariableValueType);
        assert!(error.message.contains("cell"), "{error:?}");
    }

    #[test]
    fn shared_nodes_are_visited_once_and_limits_terminate() {
        let shared = Value::Array(vec![Value::Integer(1); 4].into());
        let diamond = function(vec![shared.clone(), shared], false);
        require_eligible(&diamond, 8, 8).expect("shared nodes");

        let deep = (0..6).fold(Value::Integer(1), |inner, _| {
            Value::OptionSome(Box::new(inner))
        });
        let overflow = function(vec![deep], false);
        let depth = require_eligible(&overflow, 3, 64).expect_err("depth");
        assert_eq!(depth.kind, DebugErrorKind::EvaluationLimit);

        let wide = function(vec![Value::Integer(1); 8], false);
        let values = require_eligible(&wide, 8, 4).expect_err("values");
        assert_eq!(values.kind, DebugErrorKind::EvaluationLimit);
    }

    #[test]
    fn task_owned_cell_captures_are_counted_without_locking_payloads() {
        let cell = Arc::new(Mutex::new(Value::Integer(1)));
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = cell.lock().expect("lock");
            panic!("poison the cell");
        }));
        let owned = function(vec![Value::Cell(Arc::clone(&cell))], true);
        require_task_owned(&owned, 8, 64).expect("poisoned cell handle");
        let missing_owner =
            match Value::function(FunctionId::new(1), "work".to_string(), Vec::new()) {
                Value::Function(function) => function,
                other => panic!("expected function, got {}", other.type_name()),
            };
        assert!(
            require_task_owned(&missing_owner, 8, 64)
                .expect_err("unbound")
                .message
                .contains("task owner")
        );
        let nested = function(
            vec![Value::Array(
                vec![Value::Cell(Arc::new(Mutex::new(Value::Integer(1))))].into(),
            )],
            true,
        );
        assert!(
            require_task_owned(&nested, 8, 64)
                .expect_err("nested cell")
                .message
                .contains("cell")
        );
    }
}
