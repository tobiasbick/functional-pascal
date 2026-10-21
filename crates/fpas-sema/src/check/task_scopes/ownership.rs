//! Task scope bodies and the required lexical owner of each go expression.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`.

use fpas_diagnostics::codes::SEMA_GO_OUTSIDE_OWNING_SCOPE;
use fpas_lexer::Span;
use fpas_parser::Stmt;

use super::super::Checker;

impl Checker {
    /// Checks a `scope` body in its own lexical scope.
    pub(in crate::check) fn check_scope_stmt(&mut self, body: &[Stmt]) {
        self.task_scopes.enter_scope();
        self.scopes.push_scope();
        self.check_stmt_sequence(body);
        self.scopes.pop_scope();
        self.task_scopes.exit_scope();
    }

    /// Requires a scope opened in the routine containing the go expression.
    pub(in crate::check) fn check_go_scope_ownership(&mut self, span: Span) {
        if !self.task_scopes.owns_go() {
            self.error_with_code(
                SEMA_GO_OUTSIDE_OWNING_SCOPE,
                "`go` requires an owning `scope` in the same routine",
                "Open `scope ... end scope;` around the start, or pass an explicit TaskGroup and use StartTaskInGroup. Named and anonymous helpers do not inherit the caller's scope.",
                span,
            );
        }
    }
}
