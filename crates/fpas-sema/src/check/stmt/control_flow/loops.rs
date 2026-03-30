use super::super::super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_NON_BOOLEAN_CONDITION, SEMA_TYPE_MISMATCH};
use fpas_lexer::Span;
use fpas_parser::{Expr, Stmt, TypeExpr};

impl Checker {
    pub(in super::super) fn check_for_stmt(
        &mut self,
        var_name: &str,
        var_type: &TypeExpr,
        start: &Expr,
        end: &Expr,
        body: &Stmt,
        span: Span,
    ) {
        let var_ty = self.resolve_type_expr(var_type);
        if !var_ty.is_ordinal() && !var_ty.is_error() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "For loop variable must be an ordinal type",
                "Use integer, boolean, char, or enum.",
                span,
            );
        }

        let start_ty = self.check_expr(start);
        self.check_type_compat(&var_ty, &start_ty, "for loop start", span);

        let end_ty = self.check_expr(end);
        self.check_type_compat(&var_ty, &end_ty, "for loop end", span);

        self.scopes.push_scope();
        self.scopes.define(
            var_name,
            Symbol {
                ty: var_ty,
                mutable: false,
                kind: SymbolKind::ForVar,
            },
        );

        self.scopes.loop_depth += 1;
        self.check_stmt(body);
        self.scopes.loop_depth -= 1;

        self.scopes.pop_scope();
    }

    pub(in super::super) fn check_for_in_stmt(
        &mut self,
        var_name: &str,
        var_type: &TypeExpr,
        iterable: &Expr,
        body: &Stmt,
        span: Span,
    ) {
        let var_ty = self.resolve_type_expr(var_type);
        let iter_ty = self.check_expr(iterable);

        match &iter_ty {
            Ty::Array(elem_ty) => {
                self.check_type_compat(&var_ty, elem_ty, "for-in element", span);
            }
            // `for K: KT in <dict of KT to VT>` iterates over keys in insertion order.
            Ty::Dict(k_ty, _v_ty) => {
                self.check_type_compat(&var_ty, k_ty, "for-in dict key", span);
            }
            Ty::Error => {}
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    "For-in requires an array or dict expression",
                    "for X: T in <array of T> do ...  or  for K: K in <dict of K to V> do ...",
                    span,
                );
            }
        }

        self.scopes.push_scope();
        self.scopes.define(
            var_name,
            Symbol {
                ty: var_ty,
                mutable: false,
                kind: SymbolKind::ForVar,
            },
        );

        self.scopes.loop_depth += 1;
        self.check_stmt(body);
        self.scopes.loop_depth -= 1;

        self.scopes.pop_scope();
    }

    pub(in super::super) fn check_while_stmt(&mut self, condition: &Expr, body: &Stmt, span: Span) {
        let condition_ty = self.check_expr(condition);
        if !condition_ty.compatible_with(&Ty::Boolean) {
            self.error_with_code(
                SEMA_NON_BOOLEAN_CONDITION,
                "While condition must be a boolean expression",
                "while <boolean> do ...",
                span,
            );
        }

        self.scopes.loop_depth += 1;
        self.check_stmt(body);
        self.scopes.loop_depth -= 1;
    }

    pub(in super::super) fn check_repeat_stmt(
        &mut self,
        body: &[Stmt],
        condition: &Expr,
        span: Span,
    ) {
        self.scopes.loop_depth += 1;
        self.scopes.push_scope();
        for stmt in body {
            self.check_stmt(stmt);
        }
        self.scopes.pop_scope();
        self.scopes.loop_depth -= 1;

        let condition_ty = self.check_expr(condition);
        if !condition_ty.compatible_with(&Ty::Boolean) {
            self.error_with_code(
                SEMA_NON_BOOLEAN_CONDITION,
                "Repeat/until condition must be a boolean expression",
                "repeat ... until <boolean>",
                span,
            );
        }
    }
}
