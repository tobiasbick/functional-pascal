//! One equality capability for operators, dictionary keys and generic bounds.

use super::super::Checker;
use crate::types::Ty;

impl Checker {
    pub(crate) fn supports_equality(&self, ty: &Ty) -> bool {
        ty.supports_equality_resolving(&|ty| self.resolve_visible_type(ty))
    }
}
