//! Immutable constant bindings and compile-time expression classification.
//!
//! **Documentation:** `docs/pascal/language/basics/constants.md`

mod classification;
mod scalar;

use super::Checker;
use fpas_parser::{ConstDef, VarDef};

impl Checker {
    /// Checks an immutable program or unit constant, including computed initializers.
    pub(super) fn check_const_def(&mut self, definition: &ConstDef) {
        self.check_binding(
            &definition.name,
            &definition.type_expr,
            &definition.value,
            definition.span,
            false,
            true,
        );
    }

    /// Checks a statement-position constant without cloning its initializer AST.
    pub(crate) fn check_local_const_def(&mut self, definition: &VarDef) {
        self.check_binding(
            &definition.name,
            &definition.type_expr,
            &definition.value,
            definition.span,
            false,
            true,
        );
    }
}
