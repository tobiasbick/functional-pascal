//! Lowering of receiver calls to existing call and intrinsic operations.
//!
//! **Documentation:** `docs/pascal/language/functions/fluent-calls.md`

use fpas_ir::{Constant, IntrinsicId, Operation, TypeId, ValueId};
use fpas_parser::{Designator, Expr};
use fpas_sema::FluentCallTarget;

use super::super::context::{LoweringContext, unsupported};
use crate::CompileError;

impl LoweringContext {
    /// Lowers a designator receiver call, including mutable array operations.
    pub(super) fn lower_fluent_designator(
        &mut self,
        designator: &Designator,
        args: &[Expr],
        target: &FluentCallTarget,
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let part_count = designator.parts.len().saturating_sub(1);
        if target
            .name
            .eq_ignore_ascii_case(fpas_std::std_symbols::STD_ARRAY_PUSH)
            || target
                .name
                .eq_ignore_ascii_case(fpas_std::std_symbols::STD_ARRAY_POP)
        {
            let receiver = Designator {
                parts: designator.parts[..part_count].to_vec(),
                span: designator.span,
            };
            if target
                .name
                .eq_ignore_ascii_case(fpas_std::std_symbols::STD_ARRAY_PUSH)
            {
                let Some(value) = self.argument_for_parameter(args, 0) else {
                    return Err(unsupported(span, "fluent array push arguments"));
                };
                return self.lower_array_push_target(&receiver, value, span);
            }
            return self.lower_array_pop_target(&receiver, result, span);
        }
        let (receiver, _) =
            self.lower_member_receiver(designator, part_count, &target.receiver_reads)?;
        self.lower_fluent_value(receiver, args, target, result, span)
    }

    /// Lowers a call after its receiver value has been evaluated once.
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
        self.record_call_arguments(values.len(), span)?;

        if let Some(intrinsic) =
            crate::intrinsic_catalog::resolve(&target.name, Some(&target.receiver_ty))
        {
            self.can_spawn_tasks |= intrinsic.starts_task();
            if matches!(
                intrinsic,
                fpas_bytecode::Intrinsic::Str(fpas_bytecode::StrIntrinsic::Format)
            ) {
                let count = i64::try_from(args.len())
                    .map_err(|_| unsupported(span, "format argument count overflow"))?;
                values.push(self.emit_value(
                    Operation::Const(Constant::Integer(count)),
                    super::super::types::INTEGER,
                    span,
                )?);
            }
            let is_empty = fpas_sema::native_operation_by_implementation(&target.name)
                .is_some_and(|entry| entry.lowering == fpas_sema::NativeLowering::IsEmpty);
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
            return if is_empty {
                self.lower_native_is_empty(value, span)
            } else {
                Ok(value)
            };
        }
        Err(unsupported(span, "native catalog intrinsic"))
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
