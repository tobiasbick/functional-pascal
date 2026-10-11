//! Immutable constant bindings and compile-time expression classification.
//!
//! **Documentation:** `docs/pascal/language/basics/constants.md`

mod classification;
mod records;
mod scalar;

use super::Checker;
use fpas_parser::ConstDef;

impl Checker {
    /// Checks an immutable program or unit constant, including computed initializers.
    pub(super) fn check_const_def(&mut self, definition: &ConstDef) {
        self.check_binding(
            &definition.name,
            Some(&definition.type_expr),
            &definition.value,
            definition.span,
            false,
            true,
        );
    }
}
