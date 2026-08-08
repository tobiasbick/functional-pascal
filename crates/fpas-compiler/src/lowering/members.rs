//! Lowering for semantically resolved record methods, properties, and events.

use fpas_ir::{IrType, Operation, TypeId, ValueId};
use fpas_parser::{Designator, DesignatorPart, Expr, PostfixOperation};
use fpas_sema::{
    EventAssignedInfo, EventRaiseInfo, EventWriteInfo, MethodCallTarget, PropertyReadInfo,
    PropertyWriteInfo,
};

use crate::CompileError;

use super::context::{Callable, LoweringContext, unsupported};

impl LoweringContext {
    pub(super) fn lower_bound_method(
        &mut self,
        designator: &Designator,
        key: usize,
    ) -> Result<ValueId, CompileError> {
        let info = self
            .bound_methods
            .get(&key)
            .cloned()
            .ok_or_else(|| unsupported(designator.span, "bound method metadata"))?;
        let target = self
            .bound_method_targets
            .get(&key)
            .cloned()
            .ok_or_else(|| unsupported(designator.span, "bound method adapter"))?;
        let reads = self.property_reads.get(&key).cloned().unwrap_or_default();
        let (receiver, _) =
            self.lower_member_receiver(designator, info.receiver_part_count, &reads)?;
        self.emit_value(
            Operation::MakeClosure {
                function: target.function,
                captures: vec![receiver],
            },
            target.value_type,
            designator.span,
        )
    }

    pub(super) fn lower_property_read(
        &mut self,
        designator: &Designator,
        reads: &[PropertyReadInfo],
    ) -> Result<ValueId, CompileError> {
        self.lower_member_receiver(designator, designator.parts.len(), reads)
            .map(|(value, _)| value)
    }

    pub(super) fn lower_property_write(
        &mut self,
        target: &Designator,
        value: &Expr,
        info: &PropertyWriteInfo,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let (receiver, _) =
            self.lower_member_receiver(target, info.receiver_part_count, &info.receiver_reads)?;
        let value = self.lower_expression(value)?;
        let callable = self.member_callable(&info.setter_name, target.span, "property setter")?;
        let _ = self.emit_member_call(&callable, vec![receiver, value], span)?;
        Ok(())
    }

    pub(super) fn lower_event_write(
        &mut self,
        target: &Designator,
        value: &Expr,
        info: &EventWriteInfo,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let (receiver, _) =
            self.lower_member_receiver(target, info.receiver_part_count, &info.receiver_reads)?;
        let callable = self.member_callable(&info.setter_name, target.span, "event setter")?;
        let option_ty = callable
            .parameters
            .get(1)
            .copied()
            .ok_or_else(|| unsupported(target.span, "event setter signature"))?;
        let handler = if info.clear {
            self.emit_value(Operation::MakeNone, option_ty, span)?
        } else {
            let value = self.lower_expression(value)?;
            self.emit_value(Operation::MakeSome(value), option_ty, span)?
        };
        let _ = self.emit_member_call(&callable, vec![receiver, handler], span)?;
        Ok(())
    }

    pub(super) fn lower_event_assigned(
        &mut self,
        arguments: &[Expr],
        info: &EventAssignedInfo,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let Some(Expr::Designator(designator)) = arguments.first() else {
            return Err(unsupported(span, "Assigned event argument"));
        };
        let option = self.lower_event_getter(
            designator,
            info.receiver_part_count,
            &info.receiver_reads,
            &info.getter_name,
            span,
        )?;
        self.emit_value(Operation::IsOptionSome(option), super::types::BOOLEAN, span)
    }

    pub(super) fn lower_event_raise(
        &mut self,
        designator: &Designator,
        arguments: &[Expr],
        info: &EventRaiseInfo,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let option = self.lower_event_getter(
            designator,
            info.receiver_part_count,
            &info.receiver_reads,
            &info.getter_name,
            span,
        )?;
        let getter = self.member_callable(&info.getter_name, span, "event getter")?;
        let option_ty = getter.result;
        let handler_ty = match self.type_kind(option_ty) {
            Some(IrType::Option(handler)) => handler,
            _ => return Err(unsupported(span, "event getter result")),
        };
        let handler = self.emit_value(Operation::UnwrapSome(option), handler_ty, span)?;
        let values = arguments
            .iter()
            .map(|argument| self.lower_expression(argument))
            .collect::<Result<Vec<_>, _>>()?;
        self.record_call_arguments(values.len(), span)?;
        let result = match self.type_kind(handler_ty) {
            Some(IrType::Function { result, .. }) => result,
            _ => return Err(unsupported(span, "event handler type")),
        };
        self.emit_value(
            Operation::CallValue {
                callee: handler,
                arguments: values,
            },
            result,
            span,
        )
    }

    pub(super) fn lower_postfix_member(
        &mut self,
        value: ValueId,
        operation: &PostfixOperation,
    ) -> Result<Option<(ValueId, TypeId)>, CompileError> {
        let key = fpas_sema::postfix_operation_lookup_key(operation);
        if let Some(target) = self.bound_method_targets.get(&key).cloned() {
            let span = match operation {
                PostfixOperation::Field { span, .. }
                | PostfixOperation::MethodCall { span, .. }
                | PostfixOperation::Index { span, .. } => *span,
            };
            let closure = self.emit_value(
                Operation::MakeClosure {
                    function: target.function,
                    captures: vec![value],
                },
                target.value_type,
                span,
            )?;
            return Ok(Some((closure, target.value_type)));
        }
        match operation {
            PostfixOperation::Field { span, .. } => {
                let Some(reads) = self.property_reads.get(&key).cloned() else {
                    return Ok(None);
                };
                let Some(info) = reads.first() else {
                    return Err(unsupported(*span, "empty property metadata"));
                };
                let callable = self.member_callable(&info.getter_name, *span, "property getter")?;
                let result = self.emit_member_call(&callable, vec![value], *span)?;
                Ok(Some((result, callable.result)))
            }
            PostfixOperation::MethodCall { args, span, .. } => {
                let Some(target) = self.method_calls.get(&key).cloned() else {
                    return Ok(None);
                };
                let MethodCallTarget::Instance { qualified_name, .. } = target else {
                    return Err(unsupported(*span, "static postfix method"));
                };
                let callable = self.member_callable(&qualified_name, *span, "postfix method")?;
                let mut values = vec![value];
                values.extend(
                    args.iter()
                        .map(|argument| self.lower_expression(argument))
                        .collect::<Result<Vec<_>, _>>()?,
                );
                let result = self.emit_member_call(&callable, values, *span)?;
                Ok(Some((result, callable.result)))
            }
            PostfixOperation::Index { .. } => Ok(None),
        }
    }

    pub(super) fn member_call_result(&self, key: usize) -> Option<TypeId> {
        if let Some(target) = self.method_calls.get(&key) {
            return self
                .resolve_callable(target.qualified_name())
                .map(|item| item.result);
        }
        if let Some(info) = self.event_raises.get(&key) {
            let getter = self.resolve_callable(&info.getter_name)?;
            let IrType::Option(handler) = self.type_kind(getter.result)? else {
                return None;
            };
            let IrType::Function { result, .. } = self.type_kind(handler)? else {
                return None;
            };
            return Some(result);
        }
        None
    }

    pub(super) fn lower_method_call(
        &mut self,
        designator: &Designator,
        arguments: &[Expr],
        target: &MethodCallTarget,
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let callable = self.member_callable(target.qualified_name(), span, "record method")?;
        let mut values = Vec::new();
        if let MethodCallTarget::Instance { receiver_reads, .. } = target {
            let (receiver, _) = self.lower_member_receiver(
                designator,
                designator.parts.len().saturating_sub(1),
                receiver_reads,
            )?;
            values.push(receiver);
        }
        values.extend(
            arguments
                .iter()
                .map(|argument| self.lower_expression(argument))
                .collect::<Result<Vec<_>, _>>()?,
        );
        self.record_call_arguments(values.len(), span)?;
        self.emit_value(
            Operation::CallDirect {
                function: callable.function,
                arguments: values,
            },
            result,
            span,
        )
    }

    fn lower_event_getter(
        &mut self,
        designator: &Designator,
        receiver_part_count: usize,
        receiver_reads: &[PropertyReadInfo],
        getter_name: &str,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let (receiver, _) =
            self.lower_member_receiver(designator, receiver_part_count, receiver_reads)?;
        let getter = self.member_callable(getter_name, span, "event getter")?;
        self.emit_member_call(&getter, vec![receiver], span)
    }

    fn lower_member_receiver(
        &mut self,
        designator: &Designator,
        part_count: usize,
        reads: &[PropertyReadInfo],
    ) -> Result<(ValueId, TypeId), CompileError> {
        if part_count == 0 || part_count > designator.parts.len() {
            return Err(unsupported(designator.span, "record member receiver path"));
        }
        let mut ordered = reads.to_vec();
        ordered.sort_by_key(|info| info.receiver_part_count);
        let Some(first) = ordered.first() else {
            return self.lower_raw_designator_prefix(designator, part_count);
        };
        let (mut value, _) =
            self.lower_raw_designator_prefix(designator, first.receiver_part_count)?;
        let getter =
            self.member_callable(&first.getter_name, designator.span, "property getter")?;
        value = self.emit_member_call(&getter, vec![value], designator.span)?;
        let mut ty = getter.result;
        let mut cursor = first.receiver_part_count.saturating_add(1);
        for read in ordered.iter().skip(1) {
            (value, _) = self.lower_raw_suffix(
                value,
                ty,
                designator
                    .parts
                    .get(cursor..read.receiver_part_count)
                    .ok_or_else(|| unsupported(designator.span, "property receiver ordering"))?,
            )?;
            let getter =
                self.member_callable(&read.getter_name, designator.span, "property getter")?;
            value = self.emit_member_call(&getter, vec![value], designator.span)?;
            ty = getter.result;
            cursor = read.receiver_part_count.saturating_add(1);
        }
        self.lower_raw_suffix(
            value,
            ty,
            designator
                .parts
                .get(cursor..part_count)
                .ok_or_else(|| unsupported(designator.span, "property receiver suffix"))?,
        )
    }

    fn lower_raw_designator_prefix(
        &mut self,
        designator: &Designator,
        part_count: usize,
    ) -> Result<(ValueId, TypeId), CompileError> {
        let Some(DesignatorPart::Ident(name, _)) = designator.parts.first() else {
            return Err(unsupported(designator.span, "record member receiver"));
        };
        let ty = self
            .root_type(name)
            .ok_or_else(|| unsupported(designator.span, "record member receiver type"))?;
        let value = if self.has_binding(name) {
            self.read_named_local(name, designator.span)?
        } else {
            self.read_global(name, designator.span)?
        };
        let parts = designator
            .parts
            .get(1..part_count)
            .ok_or_else(|| unsupported(designator.span, "record member receiver prefix"))?;
        self.lower_raw_suffix(value, ty, parts)
    }

    fn lower_raw_suffix(
        &mut self,
        mut value: ValueId,
        mut ty: TypeId,
        parts: &[DesignatorPart],
    ) -> Result<(ValueId, TypeId), CompileError> {
        for part in parts {
            (value, ty) = self.lower_designator_part(value, ty, part)?;
        }
        Ok((value, ty))
    }

    fn member_callable(
        &self,
        name: &str,
        span: fpas_lexer::Span,
        kind: &str,
    ) -> Result<Callable, CompileError> {
        self.resolve_callable(name)
            .ok_or_else(|| unsupported(span, kind))
    }

    fn emit_member_call(
        &mut self,
        callable: &Callable,
        arguments: Vec<ValueId>,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        self.record_call_arguments(arguments.len(), span)?;
        self.emit_value(
            Operation::CallDirect {
                function: callable.function,
                arguments,
            },
            callable.result,
            span,
        )
    }
}
