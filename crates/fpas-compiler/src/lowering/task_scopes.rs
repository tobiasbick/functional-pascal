//! Task scope entry, normal exit, and early exits that cancel the scope's children.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`.

use fpas_ir::Operation;
use fpas_lexer::Span;
use fpas_parser::Stmt;

use crate::CompileError;

use super::context::LoweringContext;

impl LoweringContext {
    /// Lowers `scope ... end scope`; a body that falls through waits for the children.
    pub(super) fn lower_task_scope(
        &mut self,
        body: &[Stmt],
        span: Span,
    ) -> Result<(), CompileError> {
        self.emit_effect(Operation::EnterTaskScope, span)?;
        self.enter_task_scope();
        self.begin_scope();
        for statement in body {
            self.lower_statement(statement)?;
        }
        self.end_scope();
        self.exit_task_scope();
        if !self.is_terminated() {
            self.emit_effect(Operation::ExitTaskScope, span)?;
        }
        Ok(())
    }

    /// Cancels and leaves every task scope opened after `depth`, innermost first.
    pub(in crate::lowering) fn cancel_task_scopes_above(
        &mut self,
        depth: u32,
        span: Span,
    ) -> Result<(), CompileError> {
        for _ in depth..self.task_scope_depth() {
            self.emit_effect(Operation::CancelTaskScope, span)?;
        }
        Ok(())
    }
}
