//! Mutable array intrinsic lowering.
//! Documentation: `docs/pascal/language/types/array/mutating.md`.

use super::*;

impl LoweringContext {
    /// Appends through storage selected before the value argument is evaluated.
    pub(super) fn lower_array_push_target(
        &mut self,
        target: &Designator,
        value: &Expr,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let array_ty = self.mutable_array_target_type(target)?;
        let element_ty = match self.type_kind(array_ty) {
            Some(fpas_ir::IrType::Array(element)) => element,
            _ => return Err(unsupported(target.span, "mutable array target type")),
        };
        let (root, root_parts) = self.designator_root(target)?;
        let local = (root_parts == target.parts.len())
            .then(|| self.direct_local(&root))
            .flatten();
        let reference = if local.is_none() {
            let reference = self.lower_var_argument(target)?;
            Some(self.save_value(reference))
        } else {
            None
        };
        let value = self.lower_expression_as(value, element_ty)?;
        if let Some(local) = local {
            return self.emit_value(
                Operation::ArrayPush { local, value },
                super::super::types::UNIT,
                span,
            );
        }

        let reference = self.restore_value(
            reference.ok_or_else(|| unsupported(span, "array receiver reference"))?,
            span,
        )?;
        let array = self.emit_value(Operation::ReferenceRead(reference), array_ty, span)?;
        let appended = self.emit_value(Operation::MakeArray(vec![value]), array_ty, span)?;
        self.record_call_arguments(2, span)?;
        let updated = self.emit_intrinsic_value(
            fpas_bytecode::Intrinsic::Array(fpas_bytecode::ArrayIntrinsic::Concat),
            vec![array, appended],
            array_ty,
            span,
        )?;
        self.emit_effect(
            Operation::ReferenceWrite {
                reference,
                value: updated,
            },
            span,
        )?;
        self.emit_value(
            Operation::Const(Constant::Unit),
            super::super::types::UNIT,
            span,
        )
    }

    /// Pops through direct local storage or an evaluated receiver reference.
    pub(super) fn lower_array_pop_target(
        &mut self,
        target: &Designator,
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let array_ty = self.mutable_array_target_type(target)?;
        let (root, root_parts) = self.designator_root(target)?;
        if root_parts == target.parts.len()
            && let Some(local) = self.direct_local(&root)
        {
            return self.emit_value(Operation::ArrayPop { local }, result, span);
        }
        let reference = self.lower_var_argument(target)?;
        let array = self.emit_value(Operation::ReferenceRead(reference), array_ty, span)?;
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
        self.emit_effect(
            Operation::ReferenceWrite {
                reference,
                value: shortened,
            },
            span,
        )?;
        Ok(popped)
    }

    fn mutable_array_target_type(&self, target: &Designator) -> Result<TypeId, CompileError> {
        self.designator_type(target)
            .ok_or_else(|| unsupported(target.span, "mutable array target type"))
    }
}
