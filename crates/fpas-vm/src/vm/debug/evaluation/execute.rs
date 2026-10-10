//! Immutable IR walker with explicit operation and traversal budgets.
//!
//! Boolean evaluation: `docs/pascal/language/basics/operators.md#evaluation-order`.

use std::collections::HashSet;
use std::sync::{Arc, TryLockError};

use fpas_bytecode::Value;

use super::model::{DebugBinaryOperation, DebugCallTarget, DebugEvaluationLimits, DebugExpression};
use super::qualified;
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};
use crate::vm::value_ops::{self, ValueOperationError, ValueOperationErrorKind};

/// Evaluates only the operands needed by Boolean short-circuit operations.
pub(super) fn evaluate(
    expression: &DebugExpression,
    depth: usize,
    limits: DebugEvaluationLimits,
    budget: &mut EvaluationBudget,
    resolve: &mut impl FnMut(&str) -> Result<Value, DebugSessionError>,
    invoke: &mut impl FnMut(DebugCallTarget, Vec<Value>) -> Result<Value, DebugSessionError>,
) -> Result<Value, DebugSessionError> {
    evaluate_with_qualified_fallback(expression, depth, limits, budget, resolve, invoke, true)
}

fn evaluate_with_qualified_fallback(
    expression: &DebugExpression,
    depth: usize,
    limits: DebugEvaluationLimits,
    budget: &mut EvaluationBudget,
    resolve: &mut impl FnMut(&str) -> Result<Value, DebugSessionError>,
    invoke: &mut impl FnMut(DebugCallTarget, Vec<Value>) -> Result<Value, DebugSessionError>,
    allow_qualified_fallback: bool,
) -> Result<Value, DebugSessionError> {
    if depth > limits.max_depth {
        return Err(limit_error(
            format!("debug expression depth exceeds limit {}", limits.max_depth),
            "Use a shallower watch expression.",
        ));
    }
    budget.operations = budget.operations.saturating_add(1);
    if budget.operations > limits.max_operations {
        return Err(limit_error(
            format!(
                "debug expression operation count exceeds limit {}",
                limits.max_operations
            ),
            "Use a smaller watch expression.",
        ));
    }
    let value = match expression {
        DebugExpression::Integer(value) => Value::Integer(*value),
        DebugExpression::Real(value) => Value::Real(*value),
        DebugExpression::Boolean(value) => Value::Boolean(*value),
        DebugExpression::String(value) => Value::Str(value.clone().into()),
        DebugExpression::Name(name) => resolve(name)?,
        DebugExpression::VarArgument(assignment) => {
            let indexes = assignment
                .selectors
                .iter()
                .filter_map(|selector| match selector {
                    crate::vm::debug::mutation::DebugAssignmentSelector::Index(expression) => {
                        Some(expression)
                    }
                    _ => None,
                })
                .map(|expression| evaluate(expression, depth + 1, limits, budget, resolve, invoke))
                .collect::<Result<Vec<_>, _>>()?;
            return invoke(DebugCallTarget::Reference(assignment.clone()), indexes);
        }
        DebugExpression::NamedArgument { .. } => return Err(DebugSessionError {
            kind: DebugErrorKind::EvaluationType,
            message: "named debugger argument is outside a call".into(),
            hint:
                "Use named arguments only in calls of declared routines, methods, or constructors."
                    .into(),
        }),
        DebugExpression::Callable(name) => {
            return invoke(DebugCallTarget::Named(name.clone()), Vec::new());
        }
        DebugExpression::Unary { operation, operand } => {
            let operand = evaluate(operand, depth + 1, limits, budget, resolve, invoke)?;
            value_ops::unary(*operation, &operand).map_err(operation_error)?
        }
        DebugExpression::Binary {
            operation,
            left,
            right,
        } => {
            let left = evaluate(left, depth + 1, limits, budget, resolve, invoke)?;
            if matches!(
                (operation, &left),
                (DebugBinaryOperation::And, Value::Boolean(false))
                    | (DebugBinaryOperation::Or, Value::Boolean(true))
            ) {
                return Ok(left);
            }
            let right = evaluate(right, depth + 1, limits, budget, resolve, invoke)?;
            value_ops::binary(*operation, &left, &right).map_err(operation_error)?
        }
        DebugExpression::Field { base, name } => {
            count_traversal(budget, limits)?;
            let base = match evaluate_with_qualified_fallback(
                base,
                depth + 1,
                limits,
                budget,
                resolve,
                invoke,
                false,
            ) {
                Ok(value) => value,
                Err(error)
                    if error.kind == DebugErrorKind::UnknownName
                        && allow_qualified_fallback
                        && let Some(constructor) = qualified::field_name(base, name) =>
                {
                    return invoke(DebugCallTarget::Named(constructor), Vec::new());
                }
                Err(error) => return Err(error),
            };
            value_ops::field(&base, name).map_err(operation_error)?
        }
        DebugExpression::Index { base, index } => {
            count_traversal(budget, limits)?;
            let base = evaluate(base, depth + 1, limits, budget, resolve, invoke)?;
            let index = evaluate(index, depth + 1, limits, budget, resolve, invoke)?;
            value_ops::index(&base, &index).map_err(operation_error)?
        }
        DebugExpression::Call { callee, arguments } => {
            let target = match callee.as_ref() {
                DebugExpression::Callable(name) => DebugCallTarget::Named(name.clone()),
                DebugExpression::Name(name) => super::callee::resolve(name, resolve)?,
                expression => DebugCallTarget::Value(evaluate(
                    expression,
                    depth + 1,
                    limits,
                    budget,
                    resolve,
                    invoke,
                )?),
            };
            super::call_arguments::evaluate_call(
                target, arguments, depth, limits, budget, resolve, invoke,
            )?
        }
        DebugExpression::MethodCall {
            receiver,
            name,
            arguments,
        } => {
            let receiver = evaluate(receiver, depth + 1, limits, budget, resolve, invoke)?;
            super::call_arguments::evaluate_call(
                DebugCallTarget::Method {
                    receiver,
                    name: name.clone(),
                },
                arguments,
                depth,
                limits,
                budget,
                resolve,
                invoke,
            )?
        }
        DebugExpression::Array(elements) => Value::Array(
            evaluate_arguments(elements, depth, limits, budget, resolve, invoke)?.into(),
        ),
        DebugExpression::Dictionary(entries) => Value::dict(
            entries
                .iter()
                .map(|(key, value)| {
                    Ok((
                        evaluate(key, depth + 1, limits, budget, resolve, invoke)?,
                        evaluate(value, depth + 1, limits, budget, resolve, invoke)?,
                    ))
                })
                .collect::<Result<Vec<_>, DebugSessionError>>()?,
        ),
        DebugExpression::Record { name, fields } => {
            let names = fields.iter().map(|(name, _)| name.clone()).collect();
            let values = fields
                .iter()
                .map(|(_, value)| evaluate(value, depth + 1, limits, budget, resolve, invoke))
                .collect::<Result<Vec<_>, _>>()?;
            invoke(
                DebugCallTarget::Record {
                    name: name.clone(),
                    fields: names,
                },
                values,
            )?
        }
        DebugExpression::RecordUpdate { base, fields } => {
            let Value::Record(mut record) =
                evaluate(base, depth + 1, limits, budget, resolve, invoke)?
            else {
                return Err(operation_error(ValueOperationError::type_mismatch(
                    "debug record update requires a record value",
                    "Use `with` only on a record expression.",
                )));
            };
            for (name, expression) in fields {
                let Some(index) = record
                    .body()
                    .layout
                    .fields
                    .iter()
                    .position(|field| field.eq_ignore_ascii_case(name))
                else {
                    return Err(operation_error(ValueOperationError::domain(
                        format!("record has no field `{name}`"),
                        "Use a declared stored field name.",
                    )));
                };
                record.values_mut()[index] =
                    evaluate(expression, depth + 1, limits, budget, resolve, invoke)?;
            }
            Value::Record(record)
        }
        DebugExpression::ResultOk(value) => {
            Value::result_ok(evaluate(value, depth + 1, limits, budget, resolve, invoke)?)
        }
        DebugExpression::ResultError(value) => {
            Value::result_error(evaluate(value, depth + 1, limits, budget, resolve, invoke)?)
        }
        DebugExpression::OptionSome(value) => {
            Value::option_some(evaluate(value, depth + 1, limits, budget, resolve, invoke)?)
        }
        DebugExpression::OptionNone => Value::OptionNone,
        DebugExpression::If {
            branches,
            else_value,
        } => {
            // Only the selected branch value is evaluated.
            for (condition, value) in branches {
                match evaluate(condition, depth + 1, limits, budget, resolve, invoke)? {
                    Value::Boolean(true) => {
                        return evaluate(value, depth + 1, limits, budget, resolve, invoke);
                    }
                    Value::Boolean(false) => {}
                    other => {
                        return Err(operation_error(ValueOperationError::type_mismatch(
                            format!(
                                "debug `if` condition must be boolean, got {}",
                                other.type_name()
                            ),
                            "Use a boolean condition such as `Count > 0`.",
                        )));
                    }
                }
            }
            return evaluate(else_value, depth + 1, limits, budget, resolve, invoke);
        }
        DebugExpression::Try(value) => {
            match evaluate(value, depth + 1, limits, budget, resolve, invoke)? {
                Value::ResultOk(value) | Value::OptionSome(value) => value.into_inner(),
                Value::ResultError(_) => {
                    return Err(operation_error(ValueOperationError::domain(
                        "debug `try` encountered Result.Error",
                        "Inspect the error value or guard it with `Value.IsOk()`.",
                    )));
                }
                Value::OptionNone => {
                    return Err(operation_error(ValueOperationError::domain(
                        "debug `try` encountered Option.None",
                        "Inspect the option or guard it with `Value.IsSome()`.",
                    )));
                }
                other => {
                    return Err(operation_error(ValueOperationError::type_mismatch(
                        format!(
                            "debug `try` requires Result or Option, got {}",
                            other.type_name()
                        ),
                        "Apply `try` to a Result or Option expression.",
                    )));
                }
            }
        }
    };
    materialize(value, budget)
}

fn evaluate_arguments(
    expressions: &[DebugExpression],
    depth: usize,
    limits: DebugEvaluationLimits,
    budget: &mut EvaluationBudget,
    resolve: &mut impl FnMut(&str) -> Result<Value, DebugSessionError>,
    invoke: &mut impl FnMut(DebugCallTarget, Vec<Value>) -> Result<Value, DebugSessionError>,
) -> Result<Vec<Value>, DebugSessionError> {
    expressions
        .iter()
        .map(|expression| evaluate(expression, depth + 1, limits, budget, resolve, invoke))
        .collect()
}

fn materialize(
    mut value: Value,
    budget: &mut EvaluationBudget,
) -> Result<Value, DebugSessionError> {
    loop {
        let Value::Cell(cell) = value else {
            return Ok(value);
        };
        let identity = Arc::as_ptr(&cell) as usize;
        if !budget.visited_cells.insert(identity) {
            return Err(unavailable_error(
                "debug expression encountered a cyclic mutable cell",
                "Inspect the cell in Variables instead of evaluating through the cycle.",
            ));
        }
        value = match cell.try_lock() {
            Ok(inner) => inner.clone(),
            Err(TryLockError::WouldBlock) => {
                return Err(unavailable_error(
                    "debug expression value is currently locked",
                    "Retry after the value is no longer contended.",
                ));
            }
            Err(TryLockError::Poisoned(_)) => {
                return Err(unavailable_error(
                    "debug expression value is stored in a poisoned cell",
                    "Inspect the value in Variables; evaluation does not recover poisoned cells.",
                ));
            }
        };
    }
}

fn count_traversal(
    budget: &mut EvaluationBudget,
    limits: DebugEvaluationLimits,
) -> Result<(), DebugSessionError> {
    budget.traversals = budget.traversals.saturating_add(1);
    if budget.traversals > limits.max_traversals {
        return Err(limit_error(
            format!(
                "debug expression aggregate traversal count exceeds limit {}",
                limits.max_traversals
            ),
            "Use fewer field and index operations.",
        ));
    }
    Ok(())
}

fn operation_error(error: ValueOperationError) -> DebugSessionError {
    DebugSessionError {
        kind: match error.kind {
            ValueOperationErrorKind::Type => DebugErrorKind::EvaluationType,
            ValueOperationErrorKind::Domain => DebugErrorKind::EvaluationDomain,
        },
        message: error.message,
        hint: error.hint,
    }
}

fn limit_error(message: impl Into<String>, hint: impl Into<String>) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::EvaluationLimit,
        message: message.into(),
        hint: hint.into(),
    }
}

fn unavailable_error(message: impl Into<String>, hint: impl Into<String>) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::UnavailableValue,
        message: message.into(),
        hint: hint.into(),
    }
}

pub(super) struct EvaluationBudget {
    operations: usize,
    traversals: usize,
    visited_cells: HashSet<usize>,
}

impl EvaluationBudget {
    pub(super) fn new() -> Self {
        Self {
            operations: 0,
            traversals: 0,
            visited_cells: HashSet::new(),
        }
    }
}
