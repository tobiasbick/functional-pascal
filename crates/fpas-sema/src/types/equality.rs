//! Shared resolved equality capability for constraints, operators and dictionary keys.

use super::{Ty, TypeConstraint};

impl Ty {
    /// Check equality using the caller's visible-type resolver for recursive names.
    pub fn supports_equality_resolving(&self, resolve: &impl Fn(&Ty) -> Ty) -> bool {
        fn visit(ty: &Ty, resolve: &impl Fn(&Ty) -> Ty, visiting: &mut Vec<String>) -> bool {
            match resolve(ty) {
                Ty::Integer
                | Ty::Subrange(_)
                | Ty::Real
                | Ty::Boolean
                | Ty::String
                | Ty::Distinct(_)
                | Ty::Error => true,
                Ty::GenericParam(_, bound) => bound.is_some_and(|bound| {
                    matches!(
                        bound,
                        TypeConstraint::Equatable
                            | TypeConstraint::Comparable
                            | TypeConstraint::Numeric
                    )
                }),
                Ty::Option(inner) => visit(&inner, resolve, visiting),
                Ty::Result(ok, error) => {
                    visit(&ok, resolve, visiting) && visit(&error, resolve, visiting)
                }
                resolved @ (Ty::Record(_) | Ty::Enum(_)) => {
                    let name = resolved.to_string();
                    if visiting.iter().any(|seen| seen.eq_ignore_ascii_case(&name)) {
                        return true;
                    }
                    visiting.push(name);
                    let equal = match &resolved {
                        Ty::Record(record) => record
                            .fields
                            .iter()
                            .all(|(_, field)| visit(field, resolve, visiting)),
                        Ty::Enum(enumeration) => enumeration
                            .variants
                            .iter()
                            .flat_map(|variant| &variant.fields)
                            .all(|(_, field)| visit(field, resolve, visiting)),
                        _ => false,
                    };
                    visiting.pop();
                    equal
                }
                _ => false,
            }
        }
        visit(self, resolve, &mut Vec::new())
    }
}
