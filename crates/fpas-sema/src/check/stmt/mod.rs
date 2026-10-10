//! Scoped statement checking, including implicit branch and loop bodies.
//!
//! **Documentation:** `docs/pascal/language/control-flow/README.md`.

mod assignment;
mod calls;
mod control_flow;
mod event_assignment;

use super::Checker;
use fpas_parser::*;

impl Checker {
    /// Checks a statement and confines each scoped list's declarations to that list.
    pub(crate) fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Const(definition) => self.check_local_const_def(definition),
            Stmt::Null(_) => {}
            Stmt::Discard { expr, span } => self.check_discard(expr, *span),
            Stmt::Block(stmts, _) => {
                self.scopes.push_scope();
                for stmt in stmts {
                    self.check_stmt(stmt);
                }
                self.scopes.pop_scope();
            }
            Stmt::Var(var_def) => self.check_var_def(var_def, true),

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
            } => self.check_unused_call_stmt(designator, args, *span),

            Stmt::Expression { expr, span } => self.check_postfix_statement(expr, *span),

            Stmt::Go { expr, span } => {
                self.check_go_expr(expr, *span);
            }
        }
    }
}
