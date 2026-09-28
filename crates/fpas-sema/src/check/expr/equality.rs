//! Which operand types support `=` and `<>`.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md` and
//! `docs/pascal/language/types/enums.md` (from the repository root).

use super::super::Checker;
use crate::types::{Ty, TypeConstraint};

impl Checker {
    /// True when values of `ty` compare with `=` and `<>`.
    ///
    /// Scalars, strings, enums, options, and results always compare. Records and enums with
    /// payloads compare structurally when every field compares: the same variant with pairwise
    /// equal payloads, or pairwise equal fields. Arrays, dictionaries, callables, tasks, and
    /// channels never compare, so neither does an aggregate that contains one.
    pub(crate) fn supports_equality(&self, ty: &Ty) -> bool {
        if matches!(ty, Ty::Option(_) | Ty::Result(..)) || ty.is_comparable() || ty.is_ordinal() {
            return true;
        }
        self.supports_structural_equality(ty, &mut Vec::new())
    }

    fn supports_structural_equality(&self, ty: &Ty, visiting: &mut Vec<String>) -> bool {
        match self.resolve_visible_type(ty) {
            Ty::Integer | Ty::Real | Ty::Boolean | Ty::String => true,
            Ty::GenericParam(_, constraint) => matches!(
                constraint,
                Some(TypeConstraint::Comparable | TypeConstraint::Numeric)
            ),
            Ty::Option(inner) => self.supports_structural_equality(&inner, visiting),
            Ty::Result(ok, error) => {
                self.supports_structural_equality(&ok, visiting)
                    && self.supports_structural_equality(&error, visiting)
            }
            Ty::Record(record) => self.all_fields_compare(
                &record.name,
                record.fields.iter().map(|(_, field)| field),
                visiting,
            ),
            Ty::Enum(enum_ty) => self.all_fields_compare(
                &enum_ty.name,
                enum_ty
                    .variants
                    .iter()
                    .flat_map(|variant| variant.fields.iter().map(|(_, field)| field)),
                visiting,
            ),
            _ => false,
        }
    }

    /// Checks the fields of one named aggregate; a type already being checked counts as comparable.
    fn all_fields_compare<'a>(
        &self,
        name: &str,
        fields: impl Iterator<Item = &'a Ty>,
        visiting: &mut Vec<String>,
    ) -> bool {
        if visiting.iter().any(|seen| seen.eq_ignore_ascii_case(name)) {
            return true;
        }
        visiting.push(name.to_string());
        let comparable = fields
            .collect::<Vec<_>>()
            .into_iter()
            .all(|field| self.supports_structural_equality(field, visiting));
        visiting.pop();
        comparable
    }
}
