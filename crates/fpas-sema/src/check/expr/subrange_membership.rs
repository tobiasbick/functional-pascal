//! Membership in a subrange type, distinguished from membership in a value.
//!
//! **Documentation:** `docs/pascal/language/types/subranges.md`

use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::Expr;

impl Checker {
    /// Checks a type operand of `in`; value operands keep ordinary collection membership.
    pub(super) fn check_subrange_membership(
        &mut self,
        expression: &Expr,
        left_ty: &Ty,
        right_ty: &Ty,
    ) -> Option<Ty> {
        let Expr::BinaryOp { left, right, .. } = expression else {
            return None;
        };
        if left_ty.is_error() || right_ty.is_error() || !self.is_type_designator(right) {
            return None;
        }
        let Ty::Subrange(range) = right_ty else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "The type operand of `in` must be an integer subrange",
                "Use `Value in Percent` where `type Percent = 0..100;`, or supply an array, dictionary, or string value.",
                right.span(),
            );
            return Some(Ty::Error);
        };
        if !matches!(left_ty, Ty::Integer | Ty::Subrange(_)) || self.is_type_designator(left) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Membership in `{}` requires an integer or subrange value", range.name),
                "Test an integer value, for example `Input in Percent`; type names, real, Boolean, string, enum, and distinct values are not accepted.",
                left.span(),
            );
            return Some(Ty::Error);
        }
        self.subrange_memberships
            .insert(Self::expr_lookup_key(expression), range.clone());
        Some(Ty::Boolean)
    }
}
