//! Error-aware static evaluation with lazy Boolean operators.
//!
//! **Documentation:** `docs/pascal/language/basics/constants.md`.

use super::StaticConstants;
use super::conversion::{from_scalar, to_scalar};
use super::operators::{binary_operation, promote_real};
use fpas_bytecode::Value;
use fpas_ir::{Constant, UnaryOperation, constants::ConstantEvaluationError};
use fpas_lexer::Span;
use fpas_parser::{BinaryOp, Designator, Expr, UnaryOp};

/// A failed checked operation at its source expression.
pub(crate) struct StaticEvaluationError {
    /// The invalid scalar operation.
    pub(crate) operation: ConstantEvaluationError,
    /// The operator expression that cannot be evaluated.
    pub(crate) span: Span,
}

/// Checked nominal layout and omitted defaults for a static record constructor.
pub(crate) struct StaticRecord {
    /// Field names in declaration order.
    pub(crate) fields: Vec<String>,
    /// Omitted defaults in declaration order.
    pub(crate) defaults: Vec<(String, Expr)>,
}

impl StaticConstants {
    /// Evaluate reached scalar and aggregate operands with lexical resolution.
    pub(crate) fn evaluate_with(
        &self,
        expression: &Expr,
        lookup: &impl Fn(&Designator) -> Option<Value>,
        record: &impl Fn(&Expr) -> Option<StaticRecord>,
    ) -> Result<Option<Value>, StaticEvaluationError> {
        Ok(match expression {
            Expr::Integer(value, _) => Some(Value::Integer(*value)),
            Expr::Real(value, _) => Some(Value::Real(*value)),
            Expr::Bool(value, _) => Some(Value::Boolean(*value)),
            Expr::Str(value, _) => Some(Value::Str(value.clone().into())),
            Expr::Paren(inner, _) => self.evaluate_with(inner, lookup, record)?,
            Expr::Designator(designator) => lookup(designator),
            Expr::UnaryOp { op, operand, span } => {
                let Some(value) = self
                    .evaluate_with(operand, lookup, record)?
                    .as_ref()
                    .and_then(to_scalar)
                else {
                    return Ok(None);
                };
                let operation = match (op, &value) {
                    (UnaryOp::Negate, Constant::Integer(_)) => UnaryOperation::NegateInteger,
                    (UnaryOp::Negate, Constant::Real(_)) => UnaryOperation::NegateReal,
                    (UnaryOp::Not, Constant::Boolean(_)) => UnaryOperation::NotBoolean,
                    _ => return Ok(None),
                };
                fpas_ir::constants::evaluate_unary(operation, &value)
                    .map_err(|operation| StaticEvaluationError {
                        operation,
                        span: *span,
                    })?
                    .map(from_scalar)
            }
            Expr::BinaryOp {
                op,
                left,
                right,
                span,
            } => {
                let left = self.evaluate_with(left, lookup, record)?;
                if matches!(
                    (op, &left),
                    (BinaryOp::And, Some(Value::Boolean(false)))
                        | (BinaryOp::Or, Some(Value::Boolean(true)))
                ) {
                    return Ok(left);
                }
                // A missing interface value cannot establish that a lazy operand is reached.
                if left.is_none() && matches!(op, BinaryOp::And | BinaryOp::Or) {
                    return Ok(None);
                }
                let right = self.evaluate_with(right, lookup, record)?;
                let (Some(left), Some(right)) = (left, right) else {
                    return Ok(None);
                };
                match op {
                    BinaryOp::Eq => return Ok(Some(Value::Boolean(left.language_equal(&right)))),
                    BinaryOp::NotEq => {
                        return Ok(Some(Value::Boolean(!left.language_equal(&right))));
                    }
                    BinaryOp::In => {
                        return Ok(super::aggregates::membership(&left, &right).map(Value::Boolean));
                    }
                    _ => {}
                }
                let (Some(mut left), Some(mut right)) = (to_scalar(&left), to_scalar(&right))
                else {
                    return Ok(None);
                };
                if matches!(
                    (&left, &right),
                    (
                        Constant::Integer(_) | Constant::Real(_),
                        Constant::Integer(_) | Constant::Real(_)
                    )
                ) && (*op == BinaryOp::RealDiv
                    || matches!(left, Constant::Real(_))
                    || matches!(right, Constant::Real(_)))
                {
                    promote_real(&mut left);
                    promote_real(&mut right);
                }
                let Some(operation) = binary_operation(*op, &left) else {
                    return Ok(None);
                };
                fpas_ir::constants::evaluate_binary(operation, &left, &right)
                    .map_err(|operation| StaticEvaluationError {
                        operation,
                        span: *span,
                    })?
                    .map(from_scalar)
            }
            _ => self.evaluate_aggregate(expression, lookup, record)?,
        })
    }
}
