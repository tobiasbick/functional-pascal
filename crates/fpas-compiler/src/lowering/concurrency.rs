//! Register-IR lowering for retained and detached task spawning.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md`.

use fpas_ir::{Operation, ValueId};
use fpas_parser::{DesignatorPart, Expr};
use fpas_sema::{MethodCallTarget, Ty};

use crate::CompileError;

use super::context::{LoweringContext, unsupported};

impl LoweringContext {
    /// Evaluates the task callable and arguments before spawning the task.
    pub(super) fn lower_go(
        &mut self,
        expression: &Expr,
        span: fpas_lexer::Span,
        retain_result: bool,
    ) -> Result<ValueId, CompileError> {
        if let Expr::Postfix {
            base, operations, ..
        } = expression
        {
            let Some((last, prefix)) = operations.split_last() else {
                return Err(unsupported(span, "empty task postfix call"));
            };
            let key = fpas_sema::postfix_operation_lookup_key(last);
            let fpas_parser::PostfixOperation::MethodCall { args, .. } = last else {
                return Err(unsupported(span, "task postfix call"));
            };
            let receiver = if prefix.is_empty() {
                self.lower_expression(base)?
            } else {
                self.lower_postfix(base, prefix, span)?
            };
            if let Some(result_ty) = self.member_value_calls.get(&key).cloned() {
                let callee = self.lower_postfix_callable_member(receiver, last)?;
                let output = self.type_table.id(&result_ty, span.line, span.column)?;
                return self.spawn_callable_value(callee, output, args, span, retain_result);
            }
            if let Some(target) = self.fluent_calls.get(&key).cloned() {
                return self.lower_resolved_go(
                    key,
                    Some(receiver),
                    args,
                    GoTarget {
                        name: &target.name,
                        result_ty: &target.result_ty,
                    },
                    span,
                    retain_result,
                );
            }
            if let Some(target) = self.method_calls.get(&key).cloned() {
                let result_ty = self
                    .expr_types
                    .get(&fpas_sema::expr_lookup_key(expression))
                    .cloned()
                    .ok_or_else(|| unsupported(span, "task method result type"))?;
                return self.lower_resolved_go(
                    key,
                    Some(receiver),
                    args,
                    GoTarget {
                        name: target.qualified_name(),
                        result_ty: &result_ty,
                    },
                    span,
                    retain_result,
                );
            }
            return Err(unsupported(span, "task postfix receiver call"));
        }
        let Expr::Call {
            designator, args, ..
        } = expression
        else {
            return Err(unsupported(span, "invalid task expression"));
        };
        if let Some(result_ty) = self
            .member_value_calls
            .get(&fpas_sema::expr_lookup_key(expression))
            .cloned()
        {
            let key = fpas_sema::designator_lookup_key(designator);
            let callee = if let Some(reads) = self.property_reads.get(&key).cloned() {
                self.lower_property_read(designator, &reads)?
            } else {
                self.lower_designator_read(designator)?
            };
            let output = self.type_table.id(&result_ty, span.line, span.column)?;
            return self.spawn_callable_value(callee, output, args, span, retain_result);
        }
        if let Some(target) = self
            .fluent_calls
            .get(&fpas_sema::expr_lookup_key(expression))
            .cloned()
        {
            let (receiver, _) = self.lower_member_receiver(
                designator,
                designator.parts.len().saturating_sub(1),
                &target.receiver_reads,
            )?;
            return self.lower_resolved_go(
                fpas_sema::expr_lookup_key(expression),
                Some(receiver),
                args,
                GoTarget {
                    name: &target.name,
                    result_ty: &target.result_ty,
                },
                span,
                retain_result,
            );
        }
        if let Some(target) = self
            .method_calls
            .get(&fpas_sema::expr_lookup_key(expression))
            .cloned()
        {
            let receiver = match &target {
                MethodCallTarget::Instance { receiver_reads, .. } => Some(
                    self.lower_member_receiver(
                        designator,
                        designator.parts.len().saturating_sub(1),
                        receiver_reads,
                    )?
                    .0,
                ),
                MethodCallTarget::Static(_) => None,
            };
            let result_ty = self
                .expr_types
                .get(&fpas_sema::expr_lookup_key(expression))
                .cloned()
                .ok_or_else(|| unsupported(span, "task method result type"))?;
            return self.lower_resolved_go(
                fpas_sema::expr_lookup_key(expression),
                receiver,
                args,
                GoTarget {
                    name: target.qualified_name(),
                    result_ty: &result_ty,
                },
                span,
                retain_result,
            );
        }
        let [DesignatorPart::Ident(name, _)] = designator.parts.as_slice() else {
            return Err(unsupported(designator.span, "task call target"));
        };
        let (callee, output) = if self.has_binding(name) {
            let callee_ty = self
                .binding_type(name)
                .ok_or_else(|| unsupported(designator.span, "task callable binding"))?;
            let output = self
                .function_result_type(callee_ty)
                .ok_or_else(|| unsupported(designator.span, "task callable type"))?;
            (self.read_named_local(name, designator.span)?, output)
        } else {
            let callable = self
                .resolve_callable(name)
                .ok_or_else(|| unsupported(designator.span, "unresolved task call"))?;
            let captures = callable
                .captures
                .iter()
                .map(|capture| self.read_capture(&capture.name, span))
                .collect::<Result<Vec<_>, _>>()?;
            let callee = self.emit_value(
                Operation::MakeClosure {
                    function: callable.function,
                    captures,
                },
                callable.value_type,
                span,
            )?;
            (callee, callable.result)
        };
        self.spawn_callable_value(callee, output, args, span, retain_result)
    }

    fn spawn_callable_value(
        &mut self,
        callee: ValueId,
        output: fpas_ir::TypeId,
        args: &[Expr],
        span: fpas_lexer::Span,
        retain_result: bool,
    ) -> Result<ValueId, CompileError> {
        let callee = self.save_value(callee);
        let arguments = self.lower_expression_values(args, None, span)?;
        let callee = self.restore_value(callee, span)?;
        self.record_call_arguments(arguments.len(), span)?;
        self.can_spawn_tasks = true;
        if retain_result {
            let task = self.task_type(output, span)?;
            self.emit_value(Operation::SpawnTask { callee, arguments }, task, span)
        } else {
            self.emit_effect(Operation::SpawnDetachedTask { callee, arguments }, span)?;
            self.emit_value(
                Operation::Const(fpas_ir::Constant::Unit),
                super::types::UNIT,
                span,
            )
        }
    }

    fn lower_resolved_go(
        &mut self,
        key: usize,
        receiver: Option<ValueId>,
        args: &[Expr],
        target: GoTarget<'_>,
        span: fpas_lexer::Span,
        retain_result: bool,
    ) -> Result<ValueId, CompileError> {
        let GoTarget { name, result_ty } = target;
        let receiver = receiver.map(|value| self.save_value(value));
        let (callee, output) = if let Some(target) = self.intrinsic_task_targets.get(&key).cloned()
        {
            let output = self.type_table.id(result_ty, span.line, span.column)?;
            let callee = self.emit_value(
                Operation::MakeClosure {
                    function: target.function,
                    captures: Vec::new(),
                },
                target.value_type,
                span,
            )?;
            (callee, output)
        } else if self.has_binding(name) {
            let ty = self
                .binding_type(name)
                .ok_or_else(|| unsupported(span, "task callable binding"))?;
            let output = self
                .function_result_type(ty)
                .ok_or_else(|| unsupported(span, "task callable type"))?;
            (self.read_named_local(name, span)?, output)
        } else if self.has_global(name) {
            let output = self.type_table.id(result_ty, span.line, span.column)?;
            (self.read_global(name, span)?, output)
        } else {
            let callable = self
                .resolve_callable(name)
                .ok_or_else(|| unsupported(span, "fluent task callable"))?;
            let captures = callable
                .captures
                .iter()
                .map(|capture| self.read_capture(&capture.name, span))
                .collect::<Result<Vec<_>, _>>()?;
            let callee = self.emit_value(
                Operation::MakeClosure {
                    function: callable.function,
                    captures,
                },
                callable.value_type,
                span,
            )?;
            (callee, callable.result)
        };
        let callee = self.save_value(callee);
        let mut values = self.lower_expression_values(args, None, span)?;
        if let Some(receiver) = receiver {
            values.insert(0, self.restore_value(receiver, span)?);
        }
        let callee = self.restore_value(callee, span)?;
        self.record_call_arguments(values.len(), span)?;
        self.can_spawn_tasks = true;
        if retain_result {
            let task = self.task_type(output, span)?;
            self.emit_value(
                Operation::SpawnTask {
                    callee,
                    arguments: values,
                },
                task,
                span,
            )
        } else {
            self.emit_effect(
                Operation::SpawnDetachedTask {
                    callee,
                    arguments: values,
                },
                span,
            )?;
            self.emit_value(
                Operation::Const(fpas_ir::Constant::Unit),
                super::types::UNIT,
                span,
            )
        }
    }
}

/// Resolved routine started by `go`: its name and declared result type.
struct GoTarget<'a> {
    name: &'a str,
    result_ty: &'a Ty,
}
