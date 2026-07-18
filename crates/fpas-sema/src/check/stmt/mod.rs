mod calls;
mod control_flow;
mod event_assignment;
mod property_assignment;

use super::Checker;
use fpas_parser::*;

impl Checker {
    pub(crate) fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(stmts, _) => {
                self.scopes.push_scope();
                for stmt in stmts {
                    self.check_stmt(stmt);
                }
                self.scopes.pop_scope();
            }

            Stmt::Var(var_def) => self.check_var_def(var_def, false),
            Stmt::MutableVar(var_def) => self.check_var_def(var_def, true),

            Stmt::Assign {
                target,
                value,
                span,
            } => self.check_assign_stmt(target, value, *span),

            Stmt::Return(expr, span) => self.check_return_stmt(expr.as_ref(), *span),
            Stmt::Panic(expr, _) => self.check_panic_stmt(expr),

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                span,
            } => self.check_if_stmt(condition, then_branch, else_branch.as_deref(), *span),

            Stmt::Case {
                expr,
                arms,
                else_body,
                span,
            } => self.check_case_stmt(expr, arms, else_body.as_deref(), *span),

            Stmt::For {
                var_name,
                var_type,
                start,
                direction: _,
                end,
                body,
                span,
            } => self.check_for_stmt(var_name, var_type, start, end, body, *span),

            Stmt::ForIn {
                var_name,
                var_type,
                iterable,
                body,
                span,
            } => self.check_for_in_stmt(var_name, var_type, iterable, body, *span),

            Stmt::While {
                condition,
                body,
                span,
            } => self.check_while_stmt(condition, body, *span),

            Stmt::Repeat {
                body,
                condition,
                span,
            } => self.check_repeat_stmt(body, condition, *span),

            Stmt::Break(span) => self.check_break_stmt(*span),
            Stmt::Continue(span) => self.check_continue_stmt(*span),

            Stmt::Call {
                designator,
                args,
                span,
            } => self.check_call_stmt(designator, args, *span),

            Stmt::Expression { expr, span } => self.check_postfix_statement(expr, *span),

            Stmt::Go { expr, span } => {
                // `go` accepts both procedure and function calls.
                if let Expr::Call {
                    designator,
                    args,
                    span: call_span,
                } = expr
                {
                    self.check_call_stmt(designator, args, *call_span);
                    self.reject_spawned_event_raise(
                        crate::designator_lookup_key(designator),
                        *span,
                    );
                    if self.designator_refers_to_task_bound(designator) {
                        self.error_with_code(
                            fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE,
                            "Cannot spawn a task-bound callable across a task boundary",
                            "Mutable captures make a closure task-bound. Pass immutable values instead, or invoke the closure on the same task.",
                            *span,
                        );
                    }
                } else {
                    self.check_expr(expr);
                }
            }
        }
    }
}
