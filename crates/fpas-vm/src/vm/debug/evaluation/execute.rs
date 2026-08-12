//! Immutable IR walker with explicit operation and traversal budgets.

use std::collections::HashSet;
use std::sync::{Arc, TryLockError};

use fpas_bytecode::Value;

use super::model::{
    DebugBinaryOperation, DebugCallTarget, DebugEvaluationLimits, DebugExpression,
    DebugUnaryOperation,
};
use super::qualified;
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};
use crate::vm::value_ops::{
    self, BinaryOperation, UnaryOperation, ValueOperationError, ValueOperationErrorKind,
};

pub(in crate::vm::debug) fn evaluate_value(
    expression: &DebugExpression,
    limits: DebugEvaluationLimits,
    mut resolve: impl FnMut(&str) -> Result<Value, DebugSessionError>,
    mut invoke: impl FnMut(DebugCallTarget, Vec<Value>) -> Result<Value, DebugSessionError>,
) -> Result<Value, DebugSessionError> {
    let mut budget = EvaluationBudget {
        operations: 0,
        traversals: 0,
        visited_cells: HashSet::new(),
    };
    evaluate(
        expression,
        0,
        limits,
        &mut budget,
        &mut resolve,
        &mut invoke,
    )
}

/// Evaluates multiple expressions in order under one shared resource budget.
pub(in crate::vm::debug) fn evaluate_values(
    expressions: &[DebugExpression],
    limits: DebugEvaluationLimits,
    mut resolve: impl FnMut(&str) -> Result<Value, DebugSessionError>,
    mut invoke: impl FnMut(DebugCallTarget, Vec<Value>) -> Result<Value, DebugSessionError>,
) -> Result<Vec<Value>, DebugSessionError> {
    let mut budget = EvaluationBudget {
        operations: 0,
        traversals: 0,
        visited_cells: HashSet::new(),
    };
    expressions
        .iter()
        .map(|expression| {
            evaluate(
                expression,
                0,
                limits,
                &mut budget,
                &mut resolve,
                &mut invoke,
            )
        })
        .collect()
}

fn evaluate(
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
        DebugExpression::Callable(name) => {
            return invoke(DebugCallTarget::Named(name.clone()), Vec::new());
        }
        DebugExpression::Unary { operation, operand } => {
            let operand = evaluate(operand, depth + 1, limits, budget, resolve, invoke)?;
            value_ops::unary(map_unary(*operation), &operand).map_err(operation_error)?
        }
        DebugExpression::Binary {
            operation,
            left,
            right,
        } => {
            let left = evaluate(left, depth + 1, limits, budget, resolve, invoke)?;
            let right = evaluate(right, depth + 1, limits, budget, resolve, invoke)?;
            value_ops::binary(map_binary(*operation), &left, &right).map_err(operation_error)?
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
            match value_ops::field(&base, name) {
                Ok(value) => value,
                Err(_error) if matches!(base, Value::Record(_)) => invoke(
                    DebugCallTarget::Property {
                        receiver: base,
                        name: name.clone(),
                    },
                    Vec::new(),
                )?,
                Err(error) => return Err(operation_error(error)),
            }
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
                DebugExpression::Name(name) => match resolve(name) {
                    Ok(value) => DebugCallTarget::Value(value),
                    Err(error) if error.kind == DebugErrorKind::UnknownName => {
                        DebugCallTarget::Named(name.clone())
                    }
                    Err(error) => return Err(error),
                },
                expression => DebugCallTarget::Value(evaluate(
                    expression,
                    depth + 1,
                    limits,
                    budget,
                    resolve,
                    invoke,
                )?),
            };
            let arguments = evaluate_arguments(arguments, depth, limits, budget, resolve, invoke)?;
            invoke(target, arguments)?
        }
        DebugExpression::MethodCall {
            receiver,
            name,
            arguments,
        } => {
            let receiver = evaluate(receiver, depth + 1, limits, budget, resolve, invoke)?;
            let arguments = evaluate_arguments(arguments, depth, limits, budget, resolve, invoke)?;
            invoke(
                DebugCallTarget::Method {
                    receiver,
                    name: name.clone(),
                },
                arguments,
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
        DebugExpression::Record(fields) => {
            let names = fields.iter().map(|(name, _)| name.clone()).collect();
            let values = fields
                .iter()
                .map(|(_, value)| evaluate(value, depth + 1, limits, budget, resolve, invoke))
                .collect::<Result<Vec<_>, _>>()?;
            invoke(DebugCallTarget::Record { fields: names }, values)?
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
        DebugExpression::ResultOk(value) => Value::ResultOk(Box::new(evaluate(
            value,
            depth + 1,
            limits,
            budget,
            resolve,
            invoke,
        )?)),
        DebugExpression::ResultError(value) => Value::ResultError(Box::new(evaluate(
            value,
            depth + 1,
            limits,
            budget,
            resolve,
            invoke,
        )?)),
        DebugExpression::OptionSome(value) => Value::OptionSome(Box::new(evaluate(
            value,
            depth + 1,
            limits,
            budget,
            resolve,
            invoke,
        )?)),
        DebugExpression::OptionNone => Value::OptionNone,
        DebugExpression::Try(value) => {
            match evaluate(value, depth + 1, limits, budget, resolve, invoke)? {
                Value::ResultOk(value) | Value::OptionSome(value) => *value,
                Value::ResultError(_) => {
                    return Err(operation_error(ValueOperationError::domain(
                        "debug `try` encountered Result.Error",
                        "Inspect the error value or guard it with `Std.Result.IsOk`.",
                    )));
                }
                Value::OptionNone => {
                    return Err(operation_error(ValueOperationError::domain(
                        "debug `try` encountered Option.None",
                        "Inspect the option or guard it with `Std.Option.IsSome`.",
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

fn map_unary(operation: DebugUnaryOperation) -> UnaryOperation {
    match operation {
        DebugUnaryOperation::Negate => UnaryOperation::Negate,
        DebugUnaryOperation::Not => UnaryOperation::Not,
    }
}

fn map_binary(operation: DebugBinaryOperation) -> BinaryOperation {
    match operation {
        DebugBinaryOperation::Add => BinaryOperation::Add,
        DebugBinaryOperation::Subtract => BinaryOperation::Subtract,
        DebugBinaryOperation::Multiply => BinaryOperation::Multiply,
        DebugBinaryOperation::RealDivide => BinaryOperation::RealDivide,
        DebugBinaryOperation::IntegerDivide => BinaryOperation::IntegerDivide,
        DebugBinaryOperation::Modulo => BinaryOperation::Modulo,
        DebugBinaryOperation::And => BinaryOperation::And,
        DebugBinaryOperation::Or => BinaryOperation::Or,
        DebugBinaryOperation::Xor => BinaryOperation::Xor,
        DebugBinaryOperation::ShiftLeft => BinaryOperation::ShiftLeft,
        DebugBinaryOperation::ShiftRight => BinaryOperation::ShiftRight,
        DebugBinaryOperation::Equal => BinaryOperation::Equal,
        DebugBinaryOperation::NotEqual => BinaryOperation::NotEqual,
        DebugBinaryOperation::Less => BinaryOperation::Less,
        DebugBinaryOperation::LessEqual => BinaryOperation::LessEqual,
        DebugBinaryOperation::Greater => BinaryOperation::Greater,
        DebugBinaryOperation::GreaterEqual => BinaryOperation::GreaterEqual,
        DebugBinaryOperation::In => BinaryOperation::In,
    }
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

struct EvaluationBudget {
    operations: usize,
    traversals: usize,
    visited_cells: HashSet<usize>,
}
