//! Native receiver calls and explicit collection mutation.
//! Documentation: `docs/pascal/language/functions/fluent-calls.md`.

use super::super::context::{LoweringContext, unsupported};
use crate::CompileError;
use fpas_ir::{Constant, IntrinsicId, IrType, Operation, TypeId, ValueId};
use fpas_parser::{Designator, Expr};
use fpas_sema::{FluentCallTarget, NativeLowering, Ty};

impl LoweringContext {
    /// Lowers writable storage before explicit arguments, preserving written order.
    pub(super) fn lower_fluent_designator(
        &mut self,
        designator: &Designator,
        args: &[Expr],
        target: &FluentCallTarget,
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let part_count = designator.parts.len().saturating_sub(1);
        if let Some(operation) = fpas_sema::native_operation_by_implementation(&target.name)
            && let NativeLowering::InPlace(base) = operation.lowering
        {
            let receiver_ty = self
                .type_table
                .id(&target.receiver_ty, span.line, span.column)?;
            if base == fpas_builtins::std_symbols::STD_ARRAY_PUSH {
                let value = self
                    .argument_for_parameter(args, 0)
                    .ok_or_else(|| unsupported(span, "PushInPlace argument"))?;
                return self.lower_array_push_target(
                    designator,
                    part_count,
                    value,
                    receiver_ty,
                    span,
                );
            }
            if base == fpas_builtins::std_symbols::STD_ARRAY_POP {
                return self.lower_array_pop_target(
                    designator,
                    part_count,
                    result,
                    receiver_ty,
                    span,
                );
            }
            let reference = self.lower_var_argument_prefix(designator, part_count)?;
            let reference = self.save_value(reference);
            let mut values = self.lower_argument_values(args, span)?;
            let reference = self.restore_value(reference, span)?;
            let (current, storage_ty) =
                self.read_collection_reference(reference, receiver_ty, span)?;
            values.insert(0, current);
            let reference = self.save_value(reference);
            let fallible = match *operation
                .signature_for(Some(&target.receiver_ty))
                .return_type
            {
                Ty::Result(_, error) => Some(*error),
                _ => None,
            };
            let updated_type = if let Some(error) = &fallible {
                self.type_table.intern(
                    &Ty::Result(
                        Box::new(target.receiver_ty.clone()),
                        Box::new(error.clone()),
                    ),
                    span.line,
                    span.column,
                )?
            } else {
                receiver_ty
            };
            let updated = self.lower_native_values(
                base,
                &target.receiver_ty,
                values,
                updated_type,
                args.len(),
                span,
            )?;
            let reference = self.restore_value(reference, span)?;
            if fallible.is_some() {
                return self.commit_fallible_in_place(
                    reference,
                    updated,
                    updated_type,
                    storage_ty,
                    result,
                    span,
                );
            }
            let updated = self.value_as(updated, storage_ty, span)?;
            self.emit_effect(
                Operation::ReferenceWrite {
                    reference,
                    value: updated,
                },
                span,
            )?;
            return self.emit_value(
                Operation::Const(Constant::Unit),
                super::super::types::UNIT,
                span,
            );
        }
        let (receiver, _) = self.lower_member_receiver(designator, part_count)?;
        self.lower_fluent_value(receiver, args, target, result, span)
    }

    /// Commits a typed value transformation only on Ok, returning the mutation result.
    fn commit_fallible_in_place(
        &mut self,
        reference: ValueId,
        outcome: ValueId,
        outcome_type: TypeId,
        storage_type: TypeId,
        result_type: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let Some(IrType::Result { ok, error }) = self.type_kind(outcome_type) else {
            return Err(unsupported(span, "fallible InPlace value type"));
        };
        let reference_type = self
            .lowered_value_type(reference)
            .ok_or_else(|| unsupported(span, "InPlace reference type"))?;
        let saved_reference = self.declare_hidden_local(reference_type, span)?;
        self.write_local(saved_reference, reference, span)?;
        let saved_outcome = self.declare_hidden_local(outcome_type, span)?;
        self.write_local(saved_outcome, outcome, span)?;
        let output = self.declare_hidden_local(result_type, span)?;
        let success = self.new_block(span)?;
        let failure = self.new_block(span)?;
        let merge = self.new_block(span)?;
        let condition = self.emit_value(
            Operation::IsResultOk(outcome),
            super::super::types::BOOLEAN,
            span,
        )?;
        self.terminate(fpas_ir::Terminator::Branch {
            condition,
            then_target: super::super::context::target(success),
            else_target: super::super::context::target(failure),
        })?;
        self.switch_to(success);
        let outcome = self.emit_value(Operation::ReadLocal(saved_outcome), outcome_type, span)?;
        let value = self.emit_value(Operation::UnwrapOk(outcome), ok, span)?;
        let value = self.value_as(value, storage_type, span)?;
        let reference =
            self.emit_value(Operation::ReadLocal(saved_reference), reference_type, span)?;
        self.emit_effect(Operation::ReferenceWrite { reference, value }, span)?;
        let yes = self.emit_value(
            Operation::Const(Constant::Boolean(true)),
            super::super::types::BOOLEAN,
            span,
        )?;
        let value = self.emit_value(Operation::MakeOk(yes), result_type, span)?;
        self.write_local(output, value, span)?;
        self.jump(merge)?;
        self.switch_to(failure);
        let outcome = self.emit_value(Operation::ReadLocal(saved_outcome), outcome_type, span)?;
        let error_value = self.emit_value(Operation::UnwrapError(outcome), error, span)?;
        let value = self.emit_value(Operation::MakeError(error_value), result_type, span)?;
        self.write_local(output, value, span)?;
        self.jump(merge)?;
        self.switch_to(merge);
        self.emit_value(Operation::ReadLocal(output), result_type, span)
    }

    /// Lowers a value call after evaluating its receiver exactly once.
    pub(in crate::lowering) fn lower_fluent_value(
        &mut self,
        receiver: ValueId,
        args: &[Expr],
        target: &FluentCallTarget,
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let receiver = self.save_value(receiver);
        let mut values = self.lower_argument_values(args, span)?;
        values.insert(0, self.restore_value(receiver, span)?);
        self.lower_native_values(
            &target.name,
            &target.receiver_ty,
            values,
            result,
            args.len(),
            span,
        )
    }

    /// Lowers catalog value operations, including composed stack results and task bodies.
    pub(in crate::lowering) fn lower_native_values(
        &mut self,
        name: &str,
        receiver_ty: &Ty,
        mut values: Vec<ValueId>,
        result: TypeId,
        argument_count: usize,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let lowering =
            fpas_sema::native_operation_by_implementation(name).map(|entry| entry.lowering);
        if lowering == Some(NativeLowering::ArrayPush) {
            let array_ty = self.type_table.id(receiver_ty, span.line, span.column)?;
            let appended =
                self.emit_value(Operation::MakeArray(vec![values[1]]), array_ty, span)?;
            self.record_call_arguments(2, span)?;
            return self.emit_intrinsic_value(
                fpas_code::Intrinsic::Array(fpas_code::ArrayIntrinsic::Concat),
                vec![values[0], appended],
                result,
                span,
            );
        }
        if lowering == Some(NativeLowering::ArrayPop) {
            return self.lower_array_pop_value(values[0], receiver_ty, result, span);
        }
        let intrinsic = crate::intrinsic_catalog::resolve(name, Some(receiver_ty))
            .ok_or_else(|| unsupported(span, "native catalog intrinsic"))?;
        self.can_spawn_tasks |= intrinsic.starts_task();
        if matches!(
            intrinsic,
            fpas_code::Intrinsic::Str(fpas_code::StrIntrinsic::Format)
        ) {
            let count = i64::try_from(argument_count)
                .map_err(|_| unsupported(span, "format argument count overflow"))?;
            values.push(self.emit_value(
                Operation::Const(Constant::Integer(count)),
                super::super::types::INTEGER,
                span,
            )?);
        }
        self.record_call_arguments(values.len(), span)?;
        let is_empty = lowering == Some(NativeLowering::IsEmpty);
        let value = self.emit_value(
            Operation::Intrinsic {
                intrinsic: IntrinsicId::new(u32::from(u16::from(intrinsic))),
                arguments: values,
            },
            if is_empty {
                super::super::types::INTEGER
            } else {
                result
            },
            span,
        )?;
        if is_empty {
            self.lower_native_is_empty(value, span)
        } else {
            Ok(value)
        }
    }

    fn lower_array_pop_value(
        &mut self,
        array: ValueId,
        receiver_ty: &Ty,
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let array_ty = self.type_table.id(receiver_ty, span.line, span.column)?;
        let Some(IrType::Array(element)) = self.type_kind(array_ty) else {
            return Err(unsupported(span, "Pop array type"));
        };
        let (remaining, value) = self.lower_array_pop_copy(array, array_ty, element, span)?;
        let Some(IrType::Record(layout)) = self.type_kind(result) else {
            return Err(unsupported(span, "Pop result record"));
        };
        let fields = self
            .record_fields(layout)
            .ok_or_else(|| unsupported(span, "Pop fields"))?;
        let remaining = self.value_as(remaining, fields[0].1, span)?;
        let value = self.value_as(value, fields[1].1, span)?;
        self.emit_value(
            Operation::MakeRecord {
                layout,
                fields: vec![remaining, value],
            },
            result,
            span,
        )
    }

    /// Completes IsEmpty by comparing the reused length result with zero.
    pub(in crate::lowering) fn lower_native_is_empty(
        &mut self,
        length: ValueId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let zero = self.emit_value(
            Operation::Const(Constant::Integer(0)),
            super::super::types::INTEGER,
            span,
        )?;
        self.emit_binary(
            fpas_ir::BinaryOperation::Equal,
            length,
            zero,
            super::super::types::BOOLEAN,
            span,
        )
    }
}
