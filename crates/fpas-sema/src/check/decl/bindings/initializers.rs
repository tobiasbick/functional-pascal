//! Annotation checking and initializer-only local type inference.

use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::BindingDef;

impl Checker {
    /// Resolve an initializer once using only its optional expected type.
    pub(in crate::check) fn check_binding_initializer(&mut self, definition: &BindingDef) -> Ty {
        if let Some(annotation) = &definition.type_expr {
            let declared = self.resolve_type_expr(annotation);
            let actual = self.check_expr_with_expected(&definition.value, &declared);
            self.check_type_compat(&declared, &actual, "binding initializer", definition.span);
            return match (&declared, actual) {
                (Ty::Task(inner), Ty::Task(actual)) if inner.is_error() && !actual.is_error() => {
                    Ty::Task(actual)
                }
                _ => declared,
            };
        }
        let inferred = self.check_expr(&definition.value);
        if !inferred.is_error() && inferred.has_inference_holes() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Cannot infer one concrete type for binding `{}`", definition.name),
                "Add a type annotation, for example `const Values: array of (integer) := [];`. Later uses and assignments cannot supply the missing type.",
                definition.span,
            );
            Ty::Error
        } else {
            inferred
        }
    }
}
