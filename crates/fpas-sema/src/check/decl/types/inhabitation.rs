//! Finite representations for whole-unit nominal type declarations.
//!
//! **Documentation:** `docs/pascal/language/types/README.md`.

use super::Checker;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::Decl;

impl Checker {
    /// Reject mandatory cycles after every nominal header can be resolved.
    pub(super) fn validate_finite_type_headers(&mut self, declarations: &[Decl]) {
        if !self.errors.is_empty() {
            return;
        }
        for declaration in declarations {
            let Decl::TypeDef(definition) = declaration else {
                continue;
            };
            let Some(symbol) = self.scopes.lookup_root(&definition.name) else {
                continue;
            };
            if !symbol
                .ty
                .has_finite_value_with(|ty| self.resolve_visible_type(ty))
            {
                self.error_with_code(SEMA_TYPE_MISMATCH,
                    format!("Type `{}` has no finite representable value", definition.name),
                    "Break the mandatory cycle with a collection, Option, or an enum variant whose payload has a finite value.",
                    definition.span);
            }
        }
    }
}
