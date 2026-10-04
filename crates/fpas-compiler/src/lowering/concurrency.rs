//! Register-IR lowering for retained and detached task spawning.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md`.

use fpas_ir::{Operation, ValueId};
use fpas_parser::{DesignatorPart, Expr};

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
            let fpas_parser::PostfixOperation::Call { args, .. } = last else {
                return Err(unsupported(span, "task postfix call"));
            };
            let receiver = if prefix.is_empty() {
                self.lower_expression(base)?
            } else {
                self.lower_postfix(base, prefix, span)?
            };
            if let Some(target) = self.value_calls.get(&key).cloned() {
                let callable_ty =
                    self.type_table
                        .id(&target.callable_ty, span.line, span.column)?;
                let callee = self.coerce_value_type(receiver, callable_ty, span)?;
                let output = self
                    .type_table
                    .id(&target.result_ty, span.line, span.column)?;
                return self.spawn_callable_value(callee, output, args, span, retain_result);
            }
            return Err(unsupported(span, "task postfix receiver call"));
        }
        let Expr::Call {
            designator, args, ..
        } = expression
        else {
            return Err(unsupported(span, "invalid task expression"));
        };
        if let Some(target) = self
            .value_calls
            .get(&fpas_sema::expr_lookup_key(expression))
            .cloned()
        {
            let callee = self.lower_designator_read(designator)?;
            let output = self
                .type_table
                .id(&target.result_ty, span.line, span.column)?;
            return self.spawn_callable_value(callee, output, args, span, retain_result);
        }
        let name = designator
            .parts
            .iter()
            .map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(_, _) => None,
            })
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.join("."))
            .ok_or_else(|| unsupported(designator.span, "task call target"))?;
        if let Some(target) = self
            .intrinsic_task_targets
            .get(&fpas_sema::expr_lookup_key(expression))
            .cloned()
        {
            let result_ty = self
                .expr_types
                .get(&fpas_sema::expr_lookup_key(expression))
                .cloned()
                .ok_or_else(|| unsupported(span, "intrinsic task result type"))?;
            let output = self.type_table.id(&result_ty, span.line, span.column)?;
            let callee = self.emit_value(
                Operation::MakeClosure {
                    function: target.function,
                    captures: Vec::new(),
                },
                target.value_type,
                span,
            )?;
            return self.spawn_callable_value(callee, output, args, span, retain_result);
        }
        let (callee, output) = if self.has_binding(&name) || self.has_global(&name) {
            let callee_ty = self
                .root_type(&name)
                .ok_or_else(|| unsupported(designator.span, "task callable binding"))?;
            let output = self
                .function_result_type(callee_ty)
                .ok_or_else(|| unsupported(designator.span, "task callable type"))?;
            (
                if self.has_binding(&name) {
                    self.read_named_local(&name, designator.span)?
                } else {
                    self.read_global(&name, designator.span)?
                },
                output,
            )
        } else {
            let callable = self
                .resolve_callable(&name)
                .ok_or_else(|| unsupported(designator.span, "unresolved task call"))?;
            let captures = callable
                .captures
                .iter()
                .map(|capture| self.read_closure_capture(capture, span))
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
}
