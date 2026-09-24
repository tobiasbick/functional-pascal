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
                let [value] = args else {
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
        let named = self.resolve_callable(&target.name);
        let callee = if self.has_binding(&target.name) {
            let value = self.read_named_local(&target.name, span)?;
            Some(self.save_value(value))
        } else if self.has_global(&target.name) {
            let value = self.read_global(&target.name, span)?;
            Some(self.save_value(value))
        } else if let Some(callable) = &named {
            if callable.captures.is_empty() {
                None
            } else {
                let captures = callable
                    .captures
                    .iter()
                    .map(|capture| self.read_capture(&capture.name, span))
                    .collect::<Result<Vec<_>, _>>()?;
                let value = self.emit_value(
                    Operation::MakeClosure {
                        function: callable.function,
                        captures,
                    },
                    callable.value_type,
                    span,
                )?;
                Some(self.save_value(value))
            }
        } else {
            None
        };

        let mut values = self.lower_expression_values(args, None, span)?;
        values.insert(0, self.restore_value(receiver, span)?);
        let callee = callee
            .map(|value| self.restore_value(value, span))
            .transpose()?;
        self.record_call_arguments(values.len(), span)?;

        if let Some(callee) = callee {
            return self.emit_value(
                Operation::CallValue {
                    callee,
                    arguments: values,
                },
                result,
                span,
            );
        }
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
            return self.emit_value(
                Operation::Intrinsic {
                    intrinsic: IntrinsicId::new(u32::from(u16::from(intrinsic))),
                    arguments: values,
                },
                result,
                span,
            );
        }
        let callable = named.ok_or_else(|| unsupported(span, "fluent call target"))?;
        self.emit_value(
            Operation::CallDirect {
                function: callable.function,
                arguments: values,
            },
            result,
            span,
        )
    }
}
