//! Generic record and enum applications and parameter constraints.
//! See `docs/pascal/language/types/generics.md`.

use crate::{check::Checker, types::Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{QualifiedId, TypeExpr};
use std::sync::Arc;

impl Checker {
    /// Resolve the declaration, validate arity and constraints, and substitute members.
    pub(super) fn resolve_type_application(
        &mut self,
        id: &QualifiedId,
        arguments: &[TypeExpr],
        span: Span,
    ) -> Ty {
        let ty = self.resolve_type_name(id);
        let arguments: Vec<_> = arguments
            .iter()
            .map(|argument| self.resolve_type_expr(argument))
            .collect();
        let (name, parameters, applied) = match &ty {
            Ty::Record(record) => (
                &record.name,
                &record.type_params,
                !record.type_args.is_empty(),
            ),
            Ty::Enum(enumeration) => (
                &enumeration.name,
                &enumeration.type_params,
                !enumeration.type_args.is_empty(),
            ),
            _ => {
                if !ty.is_error() {
                    self.error_with_code(SEMA_TYPE_MISMATCH, "Type arguments require a generic record or enum declaration",
                    "Apply `of ...` to a declaration with parameters, such as `type Lookup of T = enum ...`.", span);
                }
                return Ty::Error;
            }
        };
        if parameters.is_empty() || applied || parameters.len() != arguments.len() {
            self.error_with_code(SEMA_TYPE_MISMATCH,
                format!("Type `{name}` expects {} type argument(s), got {}", parameters.len(), arguments.len()),
                "Use one argument after `of` or enclose multiple arguments in parentheses, matching the declaration.", span);
            return Ty::Error;
        }
        self.validate_constraints(parameters, &arguments, span);
        match ty {
            Ty::Record(record) => Ty::Record(Arc::new(record.instantiate(arguments))),
            Ty::Enum(enumeration) => Ty::Enum(Arc::new(enumeration.instantiate(arguments))),
            _ => Ty::Error,
        }
    }
}
