//! Typed and generic numeric expression lowering.
//! Documentation: docs/pascal/language/basics/operators.md.

use super::{LoweringContext, types, unsupported};
use crate::CompileError;
use fpas_ir::{BinaryOperation as IrBinary, Operation, UnaryOperation, ValueId};
use fpas_parser::Expr;
use fpas_sema::Ty;

impl LoweringContext {
    /// Lowers same-type numeric operations and real-valued generic division.
    #[expect(
        clippy::too_many_arguments,
        reason = "numeric selection keeps all typed choices explicit"
    )]
    pub(super) fn lower_numeric_binary(
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
            let output = if result_ty == types::BOOLEAN || dynamic == IrBinary::DivideDynamic {
                result_ty
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
}
