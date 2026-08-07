//! Scalar expression lowering with source-order evaluation.

use fpas_ir::{BinaryOperation as IrBinary, Constant, Operation, UnaryOperation, ValueId};
use fpas_parser::{BinaryOp, DesignatorPart, Expr, UnaryOp};
use fpas_sema::Ty;

use crate::CompileError;

use super::context::{LoweringContext, unsupported};
use super::types;

impl LoweringContext {
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
                let [DesignatorPart::Ident(name, _)] = designator.parts.as_slice() else {
                    return Err(unsupported(designator.span, "aggregate designator"));
                };
                self.read_named_local(name, designator.span)
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
        let result_ty = self.expression_ir_type(operand)?;
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
            (UnaryOp::Not, Ty::Integer) => {
                let mask = self.emit_value(
                    Operation::Const(Constant::Integer(-1)),
                    types::INTEGER,
                    span,
                )?;
                self.emit_binary(IrBinary::BitXorInteger, value, mask, result_ty, span)
            }
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
            BinaryOp::Shl | BinaryOp::Shr => self.lower_direct_binary(
                if operation == BinaryOp::Shl {
                    IrBinary::ShiftLeftInteger
                } else {
                    IrBinary::ShiftRightInteger
                },
                left,
                right,
                result_ty,
                span,
            ),
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                let ir = boolean_or_bitwise_operation(operation, &left_ty)
                    .ok_or_else(|| unsupported(span, "boolean or bitwise operation"))?;
                self.lower_direct_binary(ir, left, right, result_ty, span)
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
                let (integer, real, dynamic) = ordering_operations(operation)
                    .ok_or_else(|| unsupported(span, "ordering operation"))?;
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
            BinaryOp::In => Err(unsupported(span, "membership expression")),
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
        if matches!(left_ty, Ty::GenericParam(..)) || matches!(right_ty, Ty::GenericParam(..)) {
            return self.lower_direct_binary(dynamic, left, right, result_ty, span);
        }
        if matches!(left_ty, Ty::Real)
            || matches!(right_ty, Ty::Real)
            || integer == IrBinary::DivideReal
        {
            let (left_value, right_value) =
                self.lower_numeric_operands(left, right, left_ty, right_ty, span)?;
            return self.emit_binary(real, left_value, right_value, result_ty, span);
        }
        self.lower_direct_binary(integer, left, right, result_ty, span)
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
        let right_value = self.lower_expression(right)?;
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

    fn lower_direct_binary(
        &mut self,
        operation: IrBinary,
        left: &Expr,
        right: &Expr,
        result_ty: fpas_ir::TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let left = self.lower_expression(left)?;
        let right = self.lower_expression(right)?;
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

fn boolean_or_bitwise_operation(operation: BinaryOp, left_ty: &Ty) -> Option<IrBinary> {
    match (operation, left_ty) {
        (BinaryOp::And, Ty::Boolean) => Some(IrBinary::AndBoolean),
        (BinaryOp::Or, Ty::Boolean) => Some(IrBinary::OrBoolean),
        (BinaryOp::Xor, Ty::Boolean) => Some(IrBinary::NotEqual),
        (BinaryOp::And, _) => Some(IrBinary::BitAndInteger),
        (BinaryOp::Or, _) => Some(IrBinary::BitOrInteger),
        (BinaryOp::Xor, _) => Some(IrBinary::BitXorInteger),
        _ => None,
    }
}

fn ordering_operations(operation: BinaryOp) -> Option<(IrBinary, IrBinary, IrBinary)> {
    match operation {
        BinaryOp::Lt => Some((
            IrBinary::LessThanInteger,
            IrBinary::LessThanReal,
            IrBinary::LessThanDynamic,
        )),
        BinaryOp::Gt => Some((
            IrBinary::GreaterThanInteger,
            IrBinary::GreaterThanReal,
            IrBinary::GreaterThanDynamic,
        )),
        BinaryOp::LtEq => Some((
            IrBinary::LessEqualInteger,
            IrBinary::LessEqualReal,
            IrBinary::LessEqualDynamic,
        )),
        BinaryOp::GtEq => Some((
            IrBinary::GreaterEqualInteger,
            IrBinary::GreaterEqualReal,
            IrBinary::GreaterEqualDynamic,
        )),
        _ => None,
    }
}
