//! Nominal generic type application and constraint checking.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;

impl Checker {
    /// Apply the required arguments to a resolved type or a pending recursive header.
    pub(super) fn apply_type_arguments(&mut self, ty: Ty, arguments: Vec<Ty>, span: Span) -> Ty {
        if ty.is_error() {
            return ty;
        }
        let parameters = if let Ty::Named(name) = &ty {
            self.type_collection
                .pending_type_params(name)
                .map(|parameters| self.resolve_type_params(parameters))
                .unwrap_or_default()
        } else {
            ty.type_parameters().to_vec()
        };
        if parameters.len() != arguments.len() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Type `{ty}` requires {} type argument(s), but {} were supplied",
                    parameters.len(),
                    arguments.len()
                ),
                if parameters.is_empty() {
                    "Remove the `of (...)` list from this non-generic type.".to_owned()
                } else {
                    format!(
                        "Write `{ty} of ({})` with one explicit type per parameter.",
                        parameters
                            .iter()
                            .map(|parameter| parameter.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                },
                span,
            );
            return Ty::Error;
        }
        if arguments.is_empty() {
            return ty;
        }
        self.validate_constraints(&parameters, &arguments, span);
        if let Ty::Named(name) = ty {
            Ty::Applied(name, arguments)
        } else {
            let instantiated = ty.instantiate(&arguments).unwrap_or(Ty::Error);
            if !self.type_collection.collecting {
                self.check_nested_pure_signatures(&instantiated, span);
            }
            instantiated
        }
    }
}
