//! Register-IR lowering for retained and detached task spawning.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md`.

use fpas_ir::{Operation, ValueId};
use fpas_parser::{DesignatorPart, Expr};

use crate::CompileError;

use super::context::{LoweringContext, unsupported};

impl LoweringContext {
    pub(super) fn lower_go(
        &mut self,
        expression: &Expr,
        span: fpas_lexer::Span,
        retain_result: bool,
    ) -> Result<ValueId, CompileError> {
        let Expr::Call {
            designator, args, ..
        } = expression
        else {
            return Err(unsupported(span, "invalid task expression"));
        };
        let [DesignatorPart::Ident(name, _)] = designator.parts.as_slice() else {
            return Err(unsupported(designator.span, "task call target"));
        };
        let (callee, _output) = if self.has_binding(name) {
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
        let arguments = args
            .iter()
            .map(|argument| self.lower_expression(argument))
            .collect::<Result<Vec<_>, _>>()?;
        self.record_call_arguments(arguments.len(), span)?;
        self.can_spawn_tasks = true;
        if retain_result {
            let task = self.task_type(super::types::DYNAMIC, span)?;
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
