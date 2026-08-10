//! Immutable IR walker with explicit operation and traversal budgets.

use std::collections::HashSet;
use std::sync::{Arc, TryLockError};

use fpas_bytecode::Value;

use super::model::{
    DebugBinaryOperation, DebugEvaluationLimits, DebugExpression, DebugUnaryOperation,
};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};
use crate::vm::value_ops::{
    self, BinaryOperation, UnaryOperation, ValueOperationError, ValueOperationErrorKind,
};

pub(in crate::vm::debug) fn evaluate_value(
    expression: &DebugExpression,
    limits: DebugEvaluationLimits,
    mut resolve: impl FnMut(&str) -> Result<Value, DebugSessionError>,
) -> Result<Value, DebugSessionError> {
    let mut budget = EvaluationBudget {
        operations: 0,
        traversals: 0,
        visited_cells: HashSet::new(),
    };
    evaluate(expression, 0, limits, &mut budget, &mut resolve)
}

fn evaluate(
    expression: &DebugExpression,
    depth: usize,
    limits: DebugEvaluationLimits,
    budget: &mut EvaluationBudget,
    resolve: &mut impl FnMut(&str) -> Result<Value, DebugSessionError>,
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
        DebugExpression::Unary { operation, operand } => {
            let operand = evaluate(operand, depth + 1, limits, budget, resolve)?;
            value_ops::unary(map_unary(*operation), &operand).map_err(operation_error)?
        }
        DebugExpression::Binary {
            operation,
            left,
            right,
        } => {
            let left = evaluate(left, depth + 1, limits, budget, resolve)?;
            let right = evaluate(right, depth + 1, limits, budget, resolve)?;
            value_ops::binary(map_binary(*operation), &left, &right).map_err(operation_error)?
        }
        DebugExpression::Field { base, name } => {
            count_traversal(budget, limits)?;
            let base = evaluate(base, depth + 1, limits, budget, resolve)?;
            value_ops::field(&base, name).map_err(operation_error)?
        }
        DebugExpression::Index { base, index } => {
            count_traversal(budget, limits)?;
            let base = evaluate(base, depth + 1, limits, budget, resolve)?;
            let index = evaluate(index, depth + 1, limits, budget, resolve)?;
            value_ops::index(&base, &index).map_err(operation_error)?
        }
    };
    materialize(value, budget)
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
