//! Decision expressions: `if C then A elsif D then B else E end if`, and the branch-type
//! rule shared with `case` expressions.
//!
//! All branches share one type, checked like an assignment without numeric widening.
//! Context-typed branches such as `None` or `[]` take the type of the other branches;
//! the caller then checks the result against any expected type.
//!
//! **Documentation:** `docs/pascal/language/control-flow/if-then-else.md`

use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{Expr, IfExprBranch};

impl Checker {
    /// Checks every condition and branch value and returns their shared type.
    pub(super) fn check_if_expr(
        &mut self,
        branches: &[IfExprBranch],
        else_value: &Expr,
        expected: Option<&Ty>,
    ) -> Ty {
        let mut values = Vec::with_capacity(branches.len() + 1);
        for branch in branches {
            // `is` bindings in this condition are visible only in this branch's value.
            self.scopes.push_scope();
            self.check_branch_condition(&branch.condition, "if", branch.condition.span());
            values.push((
                self.check_expr_with_expected(&branch.value, expected),
                branch.value.span(),
            ));
            self.scopes.pop_scope();
        }
        values.push((
            self.check_expr_with_expected(else_value, expected),
            else_value.span(),
        ));
        self.shared_branch_type(values, "`if` expression branches")
    }

    /// Joins branch types that are mutually assignable, preferring the most specific one.
    ///
    /// `construct` names the branches in the mismatch message, for example
    /// "`case` expression arms".
    pub(in crate::check) fn shared_branch_type(
        &mut self,
        values: Vec<(Ty, Span)>,
        construct: &str,
    ) -> Ty {
        let mut shared: Option<Ty> = None;
        for (ty, span) in values {
            if ty.is_error() {
                continue;
            }
            let Some(current) = shared.take() else {
                shared = Some(ty);
                continue;
            };
            let left = self.resolve_visible_type(&current);
            let right = self.resolve_visible_type(&ty);
            if left.assignment_compatible_with(&right) && right.assignment_compatible_with(&left) {
                shared = Some(if placeholder_count(&right) < placeholder_count(&left) {
                    ty
                } else {
                    current
                });
                continue;
            }
            let hint = branch_mismatch_hint(&left, &right);
            self.errors.push(
                crate::error::sema_error(
                    SEMA_TYPE_MISMATCH,
                    format!("{construct} have different types: `{left}` and `{right}`"),
                    hint,
                    span,
                )
                .with_expected_found(left.to_string(), right.to_string()),
            );
            return Ty::Error;
        }
        shared.unwrap_or(Ty::Error)
    }
}

/// Counts unresolved element types such as the element of `[]` or the payload of `None`.
fn placeholder_count(ty: &Ty) -> usize {
    match ty {
        Ty::Error => 1,
        Ty::Array(inner) | Ty::Channel(inner) | Ty::Option(inner) | Ty::Task(inner) => {
            placeholder_count(inner)
        }
        Ty::Result(left, right) | Ty::Dict(left, right) => {
            placeholder_count(left) + placeholder_count(right)
        }
        _ => 0,
    }
}

fn branch_mismatch_hint(left: &Ty, right: &Ty) -> String {
    if matches!(
        (left, right),
        (Ty::Integer, Ty::Real) | (Ty::Real, Ty::Integer)
    ) {
        return "Branches are not widened. Write a real literal such as `1.0`, or convert the integer branch with `IntToReal(...)`.".to_string();
    }
    Checker::distinct_conversion_hint(left, right).unwrap_or_else(|| {
        "Give every branch the same type, or convert a branch explicitly.".to_string()
    })
}
