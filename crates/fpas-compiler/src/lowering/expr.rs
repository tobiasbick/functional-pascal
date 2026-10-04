//! Scalar expression lowering with source-order evaluation.

mod boolean;
mod decisions;
mod designators;
mod operators;

use fpas_ir::{Constant, Operation, ValueId};
use fpas_parser::Expr;

use crate::CompileError;

use super::context::{LoweringContext, unsupported};
use super::types;

impl LoweringContext {
    /// Lowers an expression to a value available in its final continuation block.
    pub(super) fn lower_expression(&mut self, expression: &Expr) -> Result<ValueId, CompileError> {
        match expression {
            Expr::If(decision) => {
                let result = self.expression_ir_type(expression)?;
                self.lower_if_expression(decision, result)
            }
            Expr::Case(decision) => {
                let result = self.expression_ir_type(expression)?;
                self.lower_case_expression(decision, result)
            }
            Expr::RecordConstruction { fields, .. } => {
                self.lower_record_construction(fields, expression)
            }
            Expr::Integer(value, span) => self.emit_value(
                Operation::Const(Constant::Integer(*value)),
                types::INTEGER,
                *span,
            ),
            Expr::Real(value, span) => {
                self.emit_value(Operation::Const(Constant::Real(*value)), types::REAL, *span)
            }
            Expr::Str(value, span) => self.emit_value(
                Operation::Const(Constant::String(value.clone())),
                types::STRING,
                *span,
            ),
            Expr::Bool(value, span) => self.emit_value(
                Operation::Const(Constant::Boolean(*value)),
                types::BOOLEAN,
                *span,
            ),
            Expr::Designator(designator) => {
                self.lower_designator_expression(designator, expression)
            }
            Expr::Call {
                designator,
                args,
                span,
            } => {
                let call_key = fpas_sema::expr_lookup_key(expression);
                if self.record_constructions.contains(&call_key) {
                    return self.lower_record_construction(&[], expression);
                }
                if let Some(info) = self.event_assigned.get(&call_key).cloned() {
                    return self.lower_event_assigned(args, &info, *span);
                }
                if let Some(info) = self.event_raises.get(&call_key).cloned() {
                    return self.lower_event_raise(designator, args, &info, *span);
                }
                let result = self.expression_ir_type(expression)?;
                self.lower_call(designator, args, result, *span, call_key)
            }
            Expr::Paren(inner, _) => self.lower_expression(inner),
            Expr::UnaryOp { op, operand, span } => self.lower_unary(*op, operand, *span),
            Expr::BinaryOp {
                op,
                left,
                right,
                span,
            } => {
                let result_ty = self.expression_ir_type(expression)?;
                self.lower_binary(*op, left, right, result_ty, *span)
            }
            Expr::Closure(_) => {
                let target = self
                    .closure_target(expression)
                    .ok_or_else(|| unsupported(expression.span(), "unregistered closure"))?;
                let captures = target
                    .captures
                    .iter()
                    .map(|capture| self.read_capture(&capture.name, expression.span()))
                    .collect::<Result<Vec<_>, _>>()?;
                self.emit_value(
                    Operation::MakeClosure {
                        function: target.function,
                        captures,
                    },
                    target.value_type,
                    expression.span(),
                )
            }
            Expr::ArrayLiteral(values, _) => self.lower_array_literal(values, expression),
            Expr::DictLiteral(values, _) => self.lower_dictionary_literal(values, expression),
            Expr::InvalidRecord(_) => Err(unsupported(expression.span(), "rejected record syntax")),
            Expr::RecordUpdate { base, fields, .. } => {
                self.lower_record_update(base, fields, expression)
            }
            Expr::ResultOk(value, _) => self.lower_wrapper(Some(value), expression, 0),
            Expr::ResultError(value, _) => self.lower_wrapper(Some(value), expression, 1),
            Expr::OptionSome(value, _) => self.lower_wrapper(Some(value), expression, 2),
            Expr::OptionNone(_) => self.lower_wrapper(None, expression, 3),
            Expr::Try(value, _) => self.lower_try(value, expression),
            Expr::Go(value, span) => self.lower_go(value, *span, true),
            Expr::Postfix {
                base,
                operations,
                span,
            } => self.lower_postfix(base, operations, *span),
            _ => Err(unsupported(expression.span(), "expression")),
        }
    }
}
