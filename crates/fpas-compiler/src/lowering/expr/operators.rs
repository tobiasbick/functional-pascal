//! Scalar operators and ordinary numeric conversions.

use super::super::context::{LoweringContext, unsupported};
use super::super::types;
use crate::CompileError;
use fpas_ir::{BinaryOperation as IrBinary, Operation, UnaryOperation, ValueId};
use fpas_parser::{BinaryOp, Expr, UnaryOp};
use fpas_sema::Ty;

impl LoweringContext {
    pub(super) fn lower_unary(
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

    pub(super) fn lower_binary(
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
                self.lower_boolean(operation, left, right, span)
            }
            BinaryOp::Eq | BinaryOp::NotEq => self.lower_numeric_comparison(
                if operation == BinaryOp::Eq {
                    IrBinary::Equal
                } else {
                    IrBinary::NotEqual
                },
                left,
                right,
                &left_ty,
                &right_ty,
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

    fn lower_numeric_comparison(
        &mut self,
        operation: IrBinary,
        left: &Expr,
        right: &Expr,
        left_ty: &Ty,
        right_ty: &Ty,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        if matches!(
            (left_ty, right_ty),
            (Ty::Integer, Ty::Real) | (Ty::Real, Ty::Integer)
        ) {
            let (left_value, right_value) =
                self.lower_numeric_operands(left, right, left_ty, right_ty, span)?;
            return self.emit_binary(operation, left_value, right_value, types::BOOLEAN, span);
        }
        self.lower_direct_binary(operation, left, right, types::BOOLEAN, span)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "numeric selection keeps all typed choices explicit"
    )]
    fn lower_numeric_binary(
        &mut self,
        integer: IrBinary,
        real: IrBinary,
        dynamic: IrBinary,
        left: &Expr,
        right: &Expr,
        left_ty: &Ty,
        right_ty: &Ty,
        result_ty: fpas_ir::TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let left_value = self.lower_expression(left)?;
        let left_value = self.save_value(left_value);
        let right_value = self.lower_expression(right)?;
        let left_value = self.restore_value(left_value, span)?;
        let left_lowered = self
            .lowered_value_type(left_value)
            .ok_or_else(|| unsupported(span, "missing lowered left operand type"))?;
        let right_lowered = self
            .lowered_value_type(right_value)
            .ok_or_else(|| unsupported(span, "missing lowered right operand type"))?;
        if left_lowered == types::DYNAMIC || right_lowered == types::DYNAMIC {
            let output = if result_ty == types::BOOLEAN {
                types::BOOLEAN
            } else {
                types::DYNAMIC
            };
            return self.emit_binary(dynamic, left_value, right_value, output, span);
        }
        if matches!(left_ty, Ty::Real)
            || matches!(right_ty, Ty::Real)
            || integer == IrBinary::DivideReal
        {
            let left_value =
                self.convert_lowered_integer_to_real(left_value, left_lowered, span)?;
            let right_value =
                self.convert_lowered_integer_to_real(right_value, right_lowered, span)?;
            return self.emit_binary(real, left_value, right_value, result_ty, span);
        }
        self.emit_binary(integer, left_value, right_value, result_ty, span)
    }

    fn convert_lowered_integer_to_real(
        &mut self,
        value: ValueId,
        ty: fpas_ir::TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        if ty == types::INTEGER {
            self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::IntegerToReal,
                    operand: value,
                },
                types::REAL,
                span,
            )
        } else {
            Ok(value)
        }
    }

    fn lower_numeric_operands(
        &mut self,
        left: &Expr,
        right: &Expr,
        left_ty: &Ty,
        right_ty: &Ty,
        span: fpas_lexer::Span,
    ) -> Result<(ValueId, ValueId), CompileError> {
        let left_value = self.lower_expression(left)?;
        let left_value = self.convert_integer_to_real(left_value, left_ty, span)?;
        let left_value = self.save_value(left_value);
        let right_value = self.lower_expression(right)?;
        let left_value = self.restore_value(left_value, span)?;
        let right_value = self.convert_integer_to_real(right_value, right_ty, span)?;
        Ok((left_value, right_value))
    }

    fn convert_integer_to_real(
        &mut self,
        value: ValueId,
        ty: &Ty,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        if matches!(ty, Ty::Integer) {
            self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::IntegerToReal,
                    operand: value,
                },
                types::REAL,
                span,
            )
        } else {
            Ok(value)
        }
    }

    pub(super) fn lower_direct_binary(
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

    pub(in crate::lowering) fn emit_binary(
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
