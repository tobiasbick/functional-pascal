//! Decision expressions: `if C then A elsif D then B else E end if`, and the branch-type
//! rule shared with `case` expressions.
//!
//! All branches share one type, checked like an assignment without numeric widening.
//! Context-typed branches such as `None` or `[]` take the type of the other branches;
//! each branch is also checked against any expected type.
//!
//! **Documentation:** `docs/pascal/language/control-flow/if-then-else.md`

use super::Checker;
use crate::types::{Ty, merge_inferred_types, needs_expected_type};
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
        let previous_errors = self.errors.len();
        self.deferred_constructor_inference += 1;
        let mut values = Vec::with_capacity(branches.len() + 1);
        for branch in branches {
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
        self.deferred_constructor_inference -= 1;
        if self.errors.len() == previous_errors
            && let Some(context) = self.branch_completion_context(&values)
        {
            return self.check_if_expr(branches, else_value, Some(&context));
        }
        self.shared_branch_type(values, "`if` expression branches", expected)
    }

    /// Finds complete sibling evidence for branches needing contextual rechecking.
    pub(in crate::check) fn branch_completion_context(&self, values: &[(Ty, Span)]) -> Option<Ty> {
        if !values
            .iter()
            .any(|(ty, _)| !ty.is_error() && needs_expected_type(ty))
        {
            return None;
        }
        if let Some((ty, _)) = values.iter().find(|(ty, _)| !needs_expected_type(ty)) {
            return Some(ty.clone());
        }
        let mut shared = Ty::Error;
        for (ty, _) in values {
            let ty = self.resolve_visible_type(ty);
            if !shared.assignment_compatible_with(&ty) || !ty.assignment_compatible_with(&shared) {
                return None;
            }
            shared = merge_inferred_types(&shared, &ty);
        }
        (!needs_expected_type(&shared)).then_some(shared)
    }

    /// Joins compatible branch evidence and checks each branch against its context.
    ///
    /// `construct` names the branches in the mismatch message, for example
    /// "`case` expression arms".
    pub(in crate::check) fn shared_branch_type(
        &mut self,
        values: Vec<(Ty, Span)>,
        construct: &str,
        expected: Option<&Ty>,
    ) -> Ty {
        let mut shared: Option<Ty> = None;
        for (ty, span) in &values {
            if ty.is_error() {
                continue;
            }
            let Some(current) = shared.take() else {
                shared = Some(ty.clone());
                continue;
            };
            let left = self.resolve_visible_type(&current);
            let right = self.resolve_visible_type(ty);
            if left.assignment_compatible_with(&right) && right.assignment_compatible_with(&left) {
                shared = Some(merge_inferred_types(&left, &right));
                continue;
            }
            let hint = branch_mismatch_hint(&left, &right);
            self.errors.push(
                crate::error::sema_error(
                    SEMA_TYPE_MISMATCH,
                    format!("{construct} have different types: `{left}` and `{right}`"),
                    hint,
                    *span,
                )
                .with_expected_found(left.to_string(), right.to_string()),
            );
            return Ty::Error;
        }
        if let Some(expected) = expected {
            for (ty, span) in &values {
                let before = self.errors.len();
                self.check_type_compat(expected, ty, construct, *span);
                if self.errors.len() != before {
                    return Ty::Error;
                }
            }
        }
        shared.unwrap_or(Ty::Error)
    }
}

fn branch_mismatch_hint(left: &Ty, right: &Ty) -> String {
    if matches!(
        (left, right),
        (Ty::Integer, Ty::Real) | (Ty::Real, Ty::Integer)
    ) {
        return "Branches are not widened. Write a real literal such as `1.0`, or import `uses Std.Conv as Conv;` and convert the integer branch with `Conv.IntToReal(...)`.".to_string();
    }
    Checker::distinct_conversion_hint(left, right).unwrap_or_else(|| {
        "Give every branch the same type, or convert a branch explicitly.".to_string()
    })
}
