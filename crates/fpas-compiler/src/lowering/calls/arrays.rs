//! Mutable array intrinsic lowering.
//! Documentation: `docs/pascal/std/collections/array/mutating.md`.

use super::*;

impl LoweringContext {
    /// Lowers array append, consuming direct local storage where available.
    pub(super) fn lower_array_push(
        &mut self,
        arguments: &[Expr],
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let [Expr::Designator(target), value] = arguments else {
            return Err(unsupported(span, "Std.Array.Push arguments"));
        };
        let array_ty = self.mutable_array_target_type(target)?;
        let element_ty = match self.type_kind(array_ty) {
            Some(fpas_ir::IrType::Array(element)) => element,
            _ => return Err(unsupported(target.span, "mutable array target type")),
        };
        let value = match value {
            Expr::RecordLiteral { fields, span } => {
                self.lower_record_literal_as(fields, element_ty, *span)?
            }
            _ => self.lower_expression(value)?,
        };
        let [DesignatorPart::Ident(name, _)] = target.parts.as_slice() else {
            return Err(unsupported(target.span, "mutable array target"));
        };
        if let Some(local) = self.direct_local(name) {
            return self.emit_value(
                Operation::ArrayPush { local, value },
                super::super::types::UNIT,
                span,
            );
        }

        let array = self.lower_designator_read(target)?;
        let appended = self.emit_value(Operation::MakeArray(vec![value]), array_ty, span)?;
        self.record_call_arguments(2, span)?;
        let updated = self.emit_intrinsic_value(
            fpas_bytecode::Intrinsic::Array(fpas_bytecode::ArrayIntrinsic::Concat),
            vec![array, appended],
            array_ty,
            span,
        )?;
        self.lower_designator_write(target, updated, span)?;
        self.emit_value(
            Operation::Const(Constant::Unit),
            super::super::types::UNIT,
            span,
        )
    }

    /// Lowers array removal while retaining the general path for cells and globals.
    pub(super) fn lower_array_pop(
        &mut self,
        arguments: &[Expr],
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let [Expr::Designator(target)] = arguments else {
            return Err(unsupported(span, "Std.Array.Pop argument"));
        };
        let array_ty = self.mutable_array_target_type(target)?;
        if let [DesignatorPart::Ident(name, _)] = target.parts.as_slice()
            && let Some(local) = self.direct_local(name)
        {
            return self.emit_value(Operation::ArrayPop { local }, result, span);
        }
        let array = self.lower_designator_read(target)?;
        self.record_call_arguments(1, span)?;
        let length = self.emit_intrinsic_value(
            fpas_bytecode::Intrinsic::Array(fpas_bytecode::ArrayIntrinsic::Length),
            vec![array],
            super::super::types::INTEGER,
            span,
        )?;
        let one = self.emit_value(
            Operation::Const(Constant::Integer(1)),
            super::super::types::INTEGER,
            span,
        )?;
        let last_index = self.emit_value(
            Operation::Binary {
                operation: fpas_ir::BinaryOperation::SubtractInteger,
                left: length,
                right: one,
            },
            super::super::types::INTEGER,
            span,
        )?;
        let popped = self.emit_value(
            Operation::IndexGet {
                collection: array,
                index: last_index,
            },
            result,
            span,
        )?;
        let zero = self.emit_value(
            Operation::Const(Constant::Integer(0)),
            super::super::types::INTEGER,
            span,
        )?;
        self.record_call_arguments(3, span)?;
        let shortened = self.emit_intrinsic_value(
            fpas_bytecode::Intrinsic::Array(fpas_bytecode::ArrayIntrinsic::Slice),
            vec![array, zero, last_index],
            array_ty,
            span,
        )?;
        self.lower_designator_write(target, shortened, span)?;
        Ok(popped)
    }

    fn mutable_array_target_type(&self, target: &Designator) -> Result<TypeId, CompileError> {
        let [DesignatorPart::Ident(name, _)] = target.parts.as_slice() else {
            return Err(unsupported(target.span, "mutable array target"));
        };
        self.root_type(name)
            .ok_or_else(|| unsupported(target.span, "mutable array target type"))
    }
}
