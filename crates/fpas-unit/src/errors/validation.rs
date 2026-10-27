//! Local declaration validation independent of compiler source ASTs.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::ErrorSchema;
use crate::interface::TypeConstraint;
use fpas_bytecode::{ErrorInstanceKey, NominalKind, TypeIdentity};
use std::collections::BTreeSet;

impl ErrorSchema {
    /// Validate a dormant generic declaration before instantiating it.
    ///
    /// # Errors
    /// Returns invalid names, bounds, payload types, labels or Cause declarations.
    pub fn validate(&self) -> Result<(), &'static str> {
        ErrorInstanceKey {
            declaration: self.declaration.clone(),
            arguments: vec![],
        }
        .validate(0)?;
        if !self.display_name.eq_ignore_ascii_case(&self.declaration) {
            return Err("error display name must identify its defining declaration");
        }
        let parameters =
            u16::try_from(self.parameters.len()).map_err(|_| "too many error schema parameters")?;
        if self.variants.is_empty()
            || self.variants.len() > fpas_bytecode::limits::MAX_ENUM_VARIANTS
        {
            return Err("error schema requires a bounded nonempty variant list");
        }
        let mut variants = BTreeSet::new();
        for variant in &self.variants {
            if !valid_member(&variant.name) || !variants.insert(variant.name.to_ascii_lowercase()) {
                return Err("invalid or duplicate error variant");
            }
            if variant.message.as_ref().is_some_and(String::is_empty) {
                return Err("error message must be omitted or nonempty");
            }
            if variant.fields.len() > fpas_bytecode::limits::MAX_LAYOUT_FIELDS {
                return Err("too many error payload fields");
            }
            let mut fields = BTreeSet::new();
            let mut cause = None;
            for (index, field) in variant.fields.iter().enumerate() {
                if !valid_member(&field.name) || !fields.insert(field.name.to_ascii_lowercase()) {
                    return Err("invalid or duplicate error payload field");
                }
                TypeIdentity::validate_all(std::slice::from_ref(&field.ty), parameters)?;
                if field.name.eq_ignore_ascii_case("Cause") {
                    cause = Some(index);
                    if !is_error(&field.ty, &self.parameters) {
                        return Err("Cause requires a native error family or Error parameter");
                    }
                } else if !printable(&field.ty, &self.parameters) {
                    return Err("error payload requires Printable");
                }
            }
            if cause != variant.cause.map(usize::from) {
                return Err("Cause slot disagrees with its field declaration");
            }
        }
        Ok(())
    }
}

pub(super) fn satisfies(
    ty: &TypeIdentity,
    bound: Option<TypeConstraint>,
    declarations: &crate::equality::EqualityDeclarations,
) -> bool {
    match bound {
        None => true,
        Some(TypeConstraint::Error) => is_error(ty, &[]),
        Some(TypeConstraint::Printable) => printable(ty, &[]),
        Some(TypeConstraint::Numeric) => matches!(ty, TypeIdentity::Integer | TypeIdentity::Real),
        Some(TypeConstraint::Equatable) => declarations.supports(ty),
        Some(TypeConstraint::Comparable) => matches!(
            ty,
            TypeIdentity::Integer
                | TypeIdentity::Real
                | TypeIdentity::Boolean
                | TypeIdentity::String
                | TypeIdentity::Subrange { .. }
                | TypeIdentity::Nominal {
                    kind: NominalKind::Distinct | NominalKind::OrdinalEnum,
                    ..
                }
        ),
    }
}

fn is_error(ty: &TypeIdentity, parameters: &[Option<TypeConstraint>]) -> bool {
    matches!(
        ty,
        TypeIdentity::Nominal {
            kind: NominalKind::Error,
            ..
        }
    ) || matches!(ty, TypeIdentity::Parameter(index) if parameters.get(usize::from(*index)) == Some(&Some(TypeConstraint::Error)))
}

fn printable(ty: &TypeIdentity, parameters: &[Option<TypeConstraint>]) -> bool {
    match ty {
        TypeIdentity::Parameter(index) => matches!(
            parameters.get(usize::from(*index)),
            Some(Some(
                TypeConstraint::Printable | TypeConstraint::Numeric | TypeConstraint::Error
            ))
        ),
        _ => ty.is_printable(),
    }
}

fn valid_member(name: &str) -> bool {
    !name.is_empty()
        && name.is_ascii()
        && name
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == b'_')
}
