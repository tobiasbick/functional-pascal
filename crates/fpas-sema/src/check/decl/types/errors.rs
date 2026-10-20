//! Native error payload admission, sharing nominal enum resolution.
//! Documentation: `docs/pascal/language/types/errors.md`.

use super::Checker;
use crate::types::{Ty, TypeConstraint};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::{Decl, TypeBody};

impl Checker {
    pub(super) fn validate_error_declarations(&mut self, declarations: &[Decl]) {
        for declaration in declarations {
            let Decl::TypeDef(definition) = declaration else {
                continue;
            };
            let TypeBody::Enum(body) = &definition.body else {
                continue;
            };
            if !body.is_error || !self.has_collected_type(definition) {
                continue;
            }
            let Some(symbol) = self.scopes.lookup_type(&definition.name) else {
                continue;
            };
            let Ty::Enum(enumeration) = symbol.ty.clone() else {
                continue;
            };
            for member in &body.members {
                let Some(variant) = enumeration
                    .variants
                    .iter()
                    .find(|variant| variant.name.eq_ignore_ascii_case(&member.name))
                else {
                    continue;
                };
                for field in &member.fields {
                    let Some((_, ty)) = variant
                        .fields
                        .iter()
                        .find(|(name, _)| name.eq_ignore_ascii_case(&field.name))
                    else {
                        continue;
                    };
                    let ty = self.resolve_visible_type(ty);
                    let cause = field.name.eq_ignore_ascii_case("Cause");
                    let constraint = if cause {
                        TypeConstraint::Error
                    } else {
                        TypeConstraint::Printable
                    };
                    if !ty.is_error() && !constraint.satisfied_by(&ty) {
                        self.error_with_code(
                            SEMA_TYPE_MISMATCH,
                            format!("Error field `{}.{}.{}` of type `{ty}` requires {}", definition.name, member.name, field.name, constraint.display_name()),
                            if cause { "Use a concrete error family or an `E: Error` parameter for Cause; omit Cause for a leaf error." }
                            else { "Use a Printable payload type, or constrain its generic parameter with `T: Printable`." },
                            field.span,
                        );
                    }
                }
            }
        }
    }
}
