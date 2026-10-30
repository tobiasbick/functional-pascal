use super::super::super::Checker;
use crate::TransferCapability;
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
        self.check_import_alias_collision(var_name, span);
        let var_ty = self.resolve_type_expr(var_type);
        if !var_ty.is_ordinal() && !var_ty.is_error() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "For loop variable must be an ordinal type",
                "Use integer, boolean, or enum.",
                span,
            );
        }

        let start_ty = self.check_expr(start);
        self.check_type_compat(&var_ty, &start_ty, "for loop start", span);

        let end_ty = self.check_expr(end);
        self.check_type_compat(&var_ty, &end_ty, "for loop end", span);

        self.check_capability_loop(true, |checker| {
            checker.scopes.push_scope();
            checker.scopes.define_with_declaration(
                var_name,
                Symbol {
                    constant: None,
                    ty: var_ty.clone(),
                    mutable: false,
                    kind: SymbolKind::ForVar,
                    capability: TransferCapability::Transferable,
                },
                span,
            );
            checker.check_stmt(body);
            checker.scopes.pop_scope();
            Self::stmt_falls_through(body)
        });
    }

    pub(in super::super) fn check_for_in_stmt(
        &mut self,
        var_name: &str,
        var_type: &TypeExpr,
        iterable: &Expr,
        body: &Stmt,
        span: Span,
    ) {
        self.check_import_alias_collision(var_name, span);
        let var_ty = self.resolve_type_expr(var_type);
        let iter_ty = self.check_expr(iterable);

        match &iter_ty {
            Ty::ByteBuffer => self.check_type_compat(&var_ty, &Ty::Integer, "for-in byte", span),
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

        let capability = if self.type_can_contain_callable(&var_ty) {
            self.expr_capability(Self::expr_lookup_key(iterable))
        } else {
            TransferCapability::Transferable
        };
        self.check_capability_loop(true, |checker| {
            checker.scopes.push_scope();
            checker.scopes.define_with_declaration(
                var_name,
                Symbol {
                    constant: None,
                    ty: var_ty.clone(),
                    mutable: false,
                    kind: SymbolKind::ForVar,
                    capability,
                },
                span,
            );
            checker.check_stmt(body);
            checker.scopes.pop_scope();
            Self::stmt_falls_through(body)
        });
    }

    pub(in super::super) fn check_while_stmt(&mut self, condition: &Expr, body: &Stmt, span: Span) {
        self.check_capability_loop(true, |checker| {
            checker.scopes.push_scope();
            checker.check_branch_condition(condition, "while", span);
            checker.check_stmt(body);
            checker.scopes.pop_scope();
            Self::stmt_falls_through(body)
        });
    }

    pub(in super::super) fn check_repeat_stmt(
        &mut self,
        body: &[Stmt],
        condition: &Expr,
        span: Span,
    ) {
        self.check_capability_loop(false, |checker| {
            checker.scopes.push_scope();
            checker.check_stmt_sequence(body);
            checker.scopes.pop_scope();
            let reaches_condition = checker
                .scopes
                .join_capability_loop_continues(Self::stmts_fall_through(body));

            let condition_ty = if reaches_condition {
                checker.check_expr(condition)
            } else {
                checker.check_unreachable(|checker| checker.check_expr(condition))
            };
            if matches!(condition_ty, Ty::GenericParam(..)) {
                checker.check_type_compat(&Ty::Boolean, &condition_ty, "repeat condition", span);
            } else if !Ty::Boolean.assignment_compatible_with(&condition_ty) {
                checker.error_with_code(
                    SEMA_NON_BOOLEAN_CONDITION,
                    "Repeat/until condition must be a boolean expression",
                    "repeat ... until <boolean>",
                    span,
                );
            }
            reaches_condition
        });
    }

    /// Check backedges to a monotone fixed point before keeping diagnostics.
    ///
    /// A later iteration can reach a transfer that precedes the assignment which
    /// introduced local state. Abrupt exits are captured before subsequent source
    /// statements can overwrite their binding facts.
    fn check_capability_loop(
        &mut self,
        may_skip: bool,
        mut check_body: impl FnMut(&mut Self) -> bool,
    ) {
        let entry = self.scopes.snapshot_capabilities();
        let mut head = entry.clone();
        let errors_before = self.errors.len();
        self.scopes.loop_depth += 1;
        loop {
            self.scopes.restore_capabilities(&head);
            self.scopes.begin_capability_loop();
            let falls_through = check_body(self);
            let flow = self.scopes.finish_capability_loop();
            let tail = self.scopes.snapshot_capabilities();
            let mut backedges = vec![head.clone()];
            if falls_through {
                backedges.push(tail.clone());
            }
            backedges.extend(flow.continues.iter().cloned());
            self.scopes.join_capability_snapshots(&backedges);
            let next = self.scopes.snapshot_capabilities();
            if next == head {
                let mut exits = Vec::new();
                if falls_through {
                    exits.push(tail);
                }
                exits.extend(flow.breaks);
                exits.extend(flow.continues);
                if may_skip {
                    exits.push(entry);
                }
                self.scopes.join_capability_snapshots(&exits);
                break;
            }
            head = next;
            self.errors.truncate(errors_before);
        }
        self.scopes.loop_depth -= 1;
    }
}
