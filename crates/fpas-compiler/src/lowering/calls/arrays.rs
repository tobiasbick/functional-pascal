//! Mutable array intrinsic lowering.
//! Documentation: `docs/pascal/language/types/array/mutating.md`.

use super::*;

impl LoweringContext {
    /// Appends through storage selected before the value argument is evaluated.
    pub(super) fn lower_array_push_target(
        &mut self,
        target: &Designator,
        value: &Expr,
        array_ty: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
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
        let (array, storage_ty) = self.read_collection_reference(reference, array_ty, span)?;
        let appended = self.emit_value(Operation::MakeArray(vec![value]), array_ty, span)?;
        self.record_call_arguments(2, span)?;
        let updated = self.emit_intrinsic_value(
            fpas_bytecode::Intrinsic::Array(fpas_bytecode::ArrayIntrinsic::Concat),
            vec![array, appended],
            array_ty,
            span,
        )?;
        let updated = self.value_as(updated, storage_ty, span)?;
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
        array_ty: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let (root, root_parts) = self.designator_root(target)?;
        let Some(fpas_ir::IrType::Array(element)) = self.type_kind(array_ty) else {
            return Err(unsupported(span, "PopInPlace element type"));
        };
        if root_parts == target.parts.len()
            && let Some(local) = self.direct_local(&root)
        {
            let popped = self.emit_value(Operation::ArrayPop { local }, element, span)?;
            return self.value_as(popped, result, span);
        }
        let reference = self.lower_var_argument(target)?;
        let (array, storage_ty) = self.read_collection_reference(reference, array_ty, span)?;
        let reference = self.save_value(reference);
        let (shortened, popped) = self.lower_array_pop_copy(array, array_ty, element, span)?;
        let reference = self.restore_value(reference, span)?;
        let shortened = self.value_as(shortened, storage_ty, span)?;
        self.emit_effect(
            Operation::ReferenceWrite {
                reference,
                value: shortened,
            },
            span,
        )?;
        self.value_as(popped, result, span)
    }

    /// Pops a private value copy using the same checked VM operation as local mutation.
    pub(super) fn lower_array_pop_copy(
        &mut self,
        array: ValueId,
        array_ty: TypeId,
        element: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<(ValueId, ValueId), CompileError> {
        let local = self.declare_hidden_local(array_ty, span)?;
        self.write_local(local, array, span)?;
        let popped = self.emit_value(Operation::ArrayPop { local }, element, span)?;
        let remaining = self.emit_value(Operation::ReadLocal(local), array_ty, span)?;
        Ok((remaining, popped))
    }
}
