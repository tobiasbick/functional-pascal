//! Lowering for semantically resolved record methods and events.

use fpas_ir::{IrType, Operation, TypeId, ValueId};
use fpas_parser::{Designator, DesignatorPart, Expr, PostfixOperation};
use fpas_sema::{EventAssignedInfo, EventRaiseInfo, EventWriteInfo, MethodCallTarget};

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
        let (receiver, _) = self.lower_member_receiver(designator, info.receiver_part_count)?;
        self.emit_value(
            Operation::MakeClosure {
                function: target.function,
                captures: vec![receiver],
            },
            target.value_type,
            designator.span,
        )
    }

    /// Retains the event receiver while evaluating the assigned handler.
    pub(super) fn lower_event_write(
        &mut self,
        target: &Designator,
        value: &Expr,
        info: &EventWriteInfo,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let (receiver, _) = self.lower_member_receiver(target, info.receiver_part_count)?;
        let receiver = self.save_value(receiver);
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
        let receiver = self.restore_value(receiver, span)?;
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
            &info.getter_name,
            span,
        )?;
        self.emit_value(Operation::IsOptionSome(option), super::types::BOOLEAN, span)
    }

    /// Retains the event handler and earlier arguments across continuations.
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
        let handler = self.save_value(handler);
        let values = self.lower_argument_values(arguments, span)?;
        let handler = self.restore_value(handler, span)?;
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

    /// Lowers a postfix member, retaining its receiver across argument evaluation.
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
            PostfixOperation::Field { .. } => Ok(None),
            PostfixOperation::MethodCall { args, span, .. } => {
                if let Some(result_ty) = self.member_value_calls.get(&key).cloned() {
                    let result = self.type_table.id(&result_ty, span.line, span.column)?;
                    let callee = self.lower_postfix_callable_member(value, operation)?;
                    let callee = self.save_value(callee);
                    let values = self.lower_argument_values(args, *span)?;
                    let callee = self.restore_value(callee, *span)?;
                    self.record_call_arguments(values.len(), *span)?;
                    let value = self.emit_value(
                        Operation::CallValue {
                            callee,
                            arguments: values,
                        },
                        result,
                        *span,
                    )?;
                    return Ok(Some((value, result)));
                }
                if let Some(target) = self.fluent_calls.get(&key).cloned() {
                    let result = self
                        .type_table
                        .id(&target.result_ty, span.line, span.column)?;
                    let value = self.lower_fluent_value(value, args, &target, result, *span)?;
                    return Ok(Some((value, result)));
                }
                let Some(target) = self.method_calls.get(&key).cloned() else {
                    return Ok(None);
                };
                let MethodCallTarget::Instance { qualified_name, .. } = target else {
                    return Err(unsupported(*span, "static postfix method"));
                };
                let callable = self.member_callable(&qualified_name, *span, "postfix method")?;
                let value = self.save_value(value);
                let mut values = self.lower_argument_values(args, *span)?;
                values.insert(0, self.restore_value(value, *span)?);
                let result = self.emit_member_call(&callable, values, *span)?;
                Ok(Some((result, callable.result)))
            }
            PostfixOperation::Index { .. } => Ok(None),
        }
    }

    /// Reads a callable field from an already evaluated record.
    pub(super) fn lower_postfix_callable_member(
        &mut self,
        value: ValueId,
        operation: &PostfixOperation,
    ) -> Result<ValueId, CompileError> {
        let PostfixOperation::MethodCall { name, span, .. } = operation else {
            unreachable!("only a method-call operation can call a record member");
        };
        let ty = self
            .lowered_value_type(value)
            .ok_or_else(|| unsupported(*span, "callable field receiver type"))?;
        self.lower_designator_part(value, ty, &DesignatorPart::Ident(name.clone(), *span))
            .map(|(callee, _)| callee)
    }

    pub(super) fn member_call_result(&self, key: usize) -> Option<TypeId> {
        if let Some(ty) = self.member_value_calls.get(&key) {
            return self.type_table.id(ty, 1, 1).ok();
        }
        if let Some(target) = self.fluent_calls.get(&key) {
            return self.type_table.id(&target.result_ty, 1, 1).ok();
        }
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

    pub(super) fn lower_member_value_call(
        &mut self,
        designator: &Designator,
        arguments: &[Expr],
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let callee = self.lower_designator_read(designator)?;
        let callee = self.save_value(callee);
        let values = self.lower_argument_values(arguments, span)?;
        let callee = self.restore_value(callee, span)?;
        self.record_call_arguments(values.len(), span)?;
        self.emit_value(
            Operation::CallValue {
                callee,
                arguments: values,
            },
            result,
            span,
        )
    }

    /// Evaluates the method receiver and arguments in source order.
    pub(super) fn lower_method_call(
        &mut self,
        designator: &Designator,
        arguments: &[Expr],
        target: &MethodCallTarget,
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let callable = self.member_callable(target.qualified_name(), span, "record method")?;
        let receiver = if let MethodCallTarget::Instance { .. } = target {
            let (receiver, _) =
                self.lower_member_receiver(designator, designator.parts.len().saturating_sub(1))?;
            Some(self.save_value(receiver))
        } else {
            None
        };
        let mut values = self.lower_argument_values(arguments, span)?;
        if let Some(receiver) = receiver {
            values.insert(0, self.restore_value(receiver, span)?);
        }
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
        getter_name: &str,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let (receiver, _) = self.lower_member_receiver(designator, receiver_part_count)?;
        let getter = self.member_callable(getter_name, span, "event getter")?;
        self.emit_member_call(&getter, vec![receiver], span)
    }

    pub(in crate::lowering) fn lower_member_receiver(
        &mut self,
        designator: &Designator,
        part_count: usize,
    ) -> Result<(ValueId, TypeId), CompileError> {
        if part_count == 0 || part_count > designator.parts.len() {
            return Err(unsupported(designator.span, "record member receiver path"));
        }
        self.lower_raw_designator_prefix(designator, part_count)
    }

    fn lower_raw_designator_prefix(
        &mut self,
        designator: &Designator,
        part_count: usize,
    ) -> Result<(ValueId, TypeId), CompileError> {
        let (name, root_parts) = self.designator_root(designator)?;
        let name = name.as_str();
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
            .get(root_parts..part_count)
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
