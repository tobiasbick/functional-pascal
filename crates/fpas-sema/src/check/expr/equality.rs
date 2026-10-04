//! Equality capability checking for scalar and aggregate expressions.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md`.

use super::super::Checker;
use crate::types::Ty;

impl Checker {
    /// Check all value components, resolving nominal recursive references.
    pub(crate) fn supports_equality(&self, ty: &Ty) -> bool {
        ty.supports_equality_with(|ty| self.resolve_visible_type(ty))
    }
}
