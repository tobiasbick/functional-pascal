//! Task scope bodies, the owner of each `go`, and results a scope cannot lose.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`.

use fpas_diagnostics::codes::{SEMA_GO_OUTSIDE_OWNING_SCOPE, SEMA_LOST_TASK_ERROR};
use fpas_lexer::Span;
use fpas_parser::{Expr, Stmt};

use super::super::Checker;
use crate::types::Ty;

impl Checker {
    /// Checks a `scope` body in its own lexical scope.
    pub(in crate::check) fn check_scope_stmt(&mut self, body: &[Stmt]) {
        self.task_scopes.enter_scope();
        self.scopes.push_scope();
        for stmt in body {
            self.check_stmt(stmt);
        }
        self.scopes.pop_scope();
        self.task_scopes.exit_scope();
    }

    /// Rejects a `go` in an anonymous routine that its lexically enclosing scope cannot own.
    pub(in crate::check) fn check_go_scope_ownership(&mut self, span: Span) {
        if self.task_scopes.go_escapes_enclosing_scope() {
            self.error_with_code(
                SEMA_GO_OUTSIDE_OWNING_SCOPE,
                "`go` in an anonymous routine cannot be owned by the enclosing `scope`",
                "Task scope ownership does not cross routine boundaries. Move the `go` into the scope body, or open `scope ... end scope;` inside the anonymous routine.",
                span,
            );
        }
    }

    /// Checks a `go` statement; inside a scope, a returned `Error` must not be dropped.
    pub(in crate::check) fn check_go_stmt(&mut self, expr: &Expr, span: Span) {
        let task = self.check_go_expr(expr, span);
        let Ty::Task(result) = &task else {
            return;
        };
        if self.task_scopes.owns_go() && matches!(result.as_ref(), Ty::Result(..)) {
            self.error_with_code(
                SEMA_LOST_TASK_ERROR,
                format!(
                    "This `go` statement inside a `scope` would lose the `Error` of its `{result}` result"
                ),
                format!(
                    "A returned `Error` does not fail the scope. Keep the handle, for example `const T: task of {result} := go ...;`, and forward the error with `try Wait(T)`."
                ),
                span,
            );
        }
    }
}
