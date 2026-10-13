//! Scalar expression lowering with source-order evaluation.

mod boolean;
mod decision;
mod designators;
mod is_test;
mod numeric;
mod subranges;

use fpas_ir::{BinaryOperation as IrBinary, Constant, Operation, UnaryOperation, ValueId};
use fpas_parser::{BinaryOp, Expr, UnaryOp};
use fpas_sema::Ty;

use crate::CompileError;

use super::context::{LoweringContext, unsupported};
use super::types;

impl LoweringContext {
    /// Lowers an expression to a value available in its final continuation block.
    pub(super) fn lower_expression(&mut self, expression: &Expr) -> Result<ValueId, CompileError> {
        match expression {
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
                let result = self.expression_ir_type(expression)?;
                self.lower_call(designator, args, result, *span, call_key)
            }
            Expr::Paren(inner, _) => self.lower_expression(inner),
            Expr::Is {
                value,
                pattern,
                span,
            } => self.lower_is_test(value, pattern, *span),
            Expr::UnaryOp { op, operand, span } => self.lower_unary(*op, operand, *span),
            Expr::BinaryOp {
                op,
                left,
                right,
                span,
            } => {
                if let Some(range) = self
                    .subrange_memberships
                    .get(&fpas_sema::expr_lookup_key(expression))
                    .cloned()
                {
                    let value = self.lower_expression(left)?;
                    return self.lower_subrange_check(value, &range, *span);
                }
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
                    .map(|capture| self.read_capture(capture, expression.span()))
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
            Expr::RecordUpdate { base, fields, .. } => {
                self.lower_record_update(base, fields, expression)
            }
            Expr::ResultOk(value, _) => self.lower_wrapper(Some(value), expression, 0),
            Expr::ResultError(value, _) => self.lower_wrapper(Some(value), expression, 1),
            Expr::OptionSome(value, _) => self.lower_wrapper(Some(value), expression, 2),
            Expr::OptionNone(_) => self.lower_wrapper(None, expression, 3),
            Expr::Try(value, _) => self.lower_try(value, expression),
            Expr::Go(value, span) => self.lower_go(value, *span),
            Expr::Postfix {
                base,
                operations,
                span,
            } => self.lower_postfix(base, operations, *span),
            Expr::NamedArgument { value, .. } => self.lower_expression(value),
            Expr::VarArgument { designator, .. } => self.lower_var_argument(designator),
            Expr::If {
                branches,
                else_value,
                ..
            } => self.lower_if_expression(branches, else_value, expression),
            Expr::Case {
                selector,
                arms,
                else_arm,
                ..
            } => self.lower_case_expression(selector, arms, else_arm.as_deref(), expression),
            _ => Err(unsupported(expression.span(), "expression")),
        }
    }

    fn lower_unary(
        &mut self,
        operation: UnaryOp,
        operand: &Expr,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let operand_ty = self.expression_type(operand)?;
        let value = self.lower_expression(operand)?;
        match (operation, operand_ty) {
            (UnaryOp::Negate, Ty::Integer) => self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::NegateInteger,
                    operand: value,
                },
                types::INTEGER,
                span,
            ),
            (UnaryOp::Negate, Ty::Real) => self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::NegateReal,
                    operand: value,
                },
                types::REAL,
                span,
            ),
            (UnaryOp::Negate, Ty::GenericParam(..)) => self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::NegateDynamic,
                    operand: value,
                },
                types::DYNAMIC,
                span,
            ),
            (UnaryOp::Not, Ty::Boolean) => self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::NotBoolean,
                    operand: value,
                },
                types::BOOLEAN,
                span,
            ),
            _ => Err(unsupported(span, "unary operand type")),
        }
    }

    fn lower_binary(
        &mut self,
        operation: BinaryOp,
        left: &Expr,
        right: &Expr,
        result_ty: fpas_ir::TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let left_ty = self.expression_type(left)?;
        let right_ty = self.expression_type(right)?;
        match operation {
            BinaryOp::Add => {
                if matches!((&left_ty, &right_ty), (Ty::String, Ty::String)) {
                    return self.lower_direct_binary(
                        IrBinary::ConcatString,
                        left,
                        right,
                        types::STRING,
                        span,
                    );
                }
                self.lower_numeric_binary(
                    IrBinary::AddInteger,
                    IrBinary::AddReal,
                    IrBinary::AddDynamic,
                    left,
                    right,
                    &left_ty,
                    &right_ty,
                    result_ty,
                    span,
                )
            }
            BinaryOp::Sub => self.lower_numeric_binary(
                IrBinary::SubtractInteger,
                IrBinary::SubtractReal,
                IrBinary::SubtractDynamic,
                left,
                right,
                &left_ty,
                &right_ty,
                result_ty,
                span,
            ),
            BinaryOp::Mul => self.lower_numeric_binary(
                IrBinary::MultiplyInteger,
                IrBinary::MultiplyReal,
                IrBinary::MultiplyDynamic,
                left,
                right,
                &left_ty,
                &right_ty,
                result_ty,
                span,
            ),
            BinaryOp::RealDiv => self.lower_numeric_binary(
                IrBinary::DivideReal,
                IrBinary::DivideReal,
                IrBinary::DivideDynamic,
                left,
                right,
                &left_ty,
                &right_ty,
                result_ty,
                span,
            ),
            BinaryOp::IntDiv => {
                self.lower_direct_binary(IrBinary::DivideInteger, left, right, result_ty, span)
            }
            BinaryOp::Mod => {
                self.lower_direct_binary(IrBinary::RemainderInteger, left, right, result_ty, span)
            }
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                self.lower_boolean(operation, left, right, result_ty, span)
            }
            BinaryOp::Eq | BinaryOp::NotEq => self.lower_direct_binary(
                if operation == BinaryOp::Eq {
                    IrBinary::Equal
                } else {
                    IrBinary::NotEqual
                },
                left,
                right,
                types::BOOLEAN,
                span,
            ),
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                let (integer, real, string, dynamic) = ordering_operations(operation)
                    .ok_or_else(|| unsupported(span, "ordering operation"))?;
                if matches!((&left_ty, &right_ty), (Ty::String, Ty::String)) {
                    self.lower_direct_binary(string, left, right, types::BOOLEAN, span)
                } else {
                    self.lower_numeric_binary(
                        integer,
                        real,
                        dynamic,
                        left,
                        right,
                        &left_ty,
                        &right_ty,
                        types::BOOLEAN,
                        span,
                    )
                }
            }
            BinaryOp::In => {
                let value = self.lower_expression(left)?;
                let value = self.save_value(value);
                let collection = self.lower_expression(right)?;
                let value = self.restore_value(value, span)?;
                self.emit_value(
                    Operation::Contains { value, collection },
                    types::BOOLEAN,
                    span,
                )
            }
        }
    }

    fn lower_direct_binary(
        &mut self,
        operation: IrBinary,
        left: &Expr,
        right: &Expr,
        result_ty: fpas_ir::TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let left = self.lower_expression(left)?;
        let left = self.save_value(left);
        let right = self.lower_expression(right)?;
        let left = self.restore_value(left, span)?;
        self.emit_binary(operation, left, right, result_ty, span)
    }

    pub(super) fn emit_binary(
        &mut self,
        operation: IrBinary,
        left: ValueId,
        right: ValueId,
        result_ty: fpas_ir::TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        self.emit_value(
            Operation::Binary {
                operation,
                left,
                right,
            },
            result_ty,
            span,
        )
    }
}

fn ordering_operations(operation: BinaryOp) -> Option<(IrBinary, IrBinary, IrBinary, IrBinary)> {
    match operation {
        BinaryOp::Lt => Some((
            IrBinary::LessThanInteger,
            IrBinary::LessThanReal,
            IrBinary::LessThanString,
            IrBinary::LessThanDynamic,
        )),
        BinaryOp::Gt => Some((
            IrBinary::GreaterThanInteger,
            IrBinary::GreaterThanReal,
            IrBinary::GreaterThanString,
            IrBinary::GreaterThanDynamic,
        )),
        BinaryOp::LtEq => Some((
            IrBinary::LessEqualInteger,
            IrBinary::LessEqualReal,
            IrBinary::LessEqualString,
            IrBinary::LessEqualDynamic,
        )),
        BinaryOp::GtEq => Some((
            IrBinary::GreaterEqualInteger,
            IrBinary::GreaterEqualReal,
            IrBinary::GreaterEqualString,
            IrBinary::GreaterEqualDynamic,
        )),
        _ => None,
    }
}
