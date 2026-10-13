//! Nonbinding Boolean pattern tests.
//!
//! Documentation: `docs/pascal/language/pattern-matching/is-test.md`.

use super::Checker;
use crate::types::Ty;
use fpas_parser::{Expr, Pattern};

impl Checker {
    /// Checks a pattern predicate without introducing any names into its scope.
    pub(super) fn check_is_test(&mut self, value: &Expr, pattern: &Pattern) -> Ty {
        let value_ty = self.check_expr(value);
        self.check_is_pattern(&value_ty, pattern);
        Ty::Boolean
    }
}
