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
        self.exit_scopes.push(super::cleanup::ExitScope::Tasks);
        self.begin_scope();
        self.lower_statements(body)?;
        self.end_scope();
        self.exit_scopes.pop();
        if !self.is_terminated() {
            self.emit_effect(Operation::ExitTaskScope, span)?;
        }
        Ok(())
    }
}
