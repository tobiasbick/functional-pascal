//! Native error declarations transported independently of implementation source.
//! Documentation: `docs/pascal/language/types/errors.md`.

use super::{GenericParamDef, Ty, semantic_type_identity};
use crate::{EnumTy, TypeConstraint};
use fpas_unit::{
    errors::{ErrorSchema, ErrorSchemaField, ErrorSchemaVariant},
    interface,
};

/// Retain the complete checked declaration of an error family.
///
/// # Errors
/// Rejects unresolved payload identities or an invalid schema.
pub fn error_schema(
    enumeration: &EnumTy,
    owner: &str,
    resolve: &impl Fn(&str) -> Option<Ty>,
) -> Result<ErrorSchema, String> {
    let parameters: &[GenericParamDef] = &enumeration.type_params;
    let identity = semantic_type_identity(
        &Ty::Enum(std::sync::Arc::new(enumeration.clone())),
        owner,
        parameters,
        resolve,
    )?;
    let key = identity
        .error_key()
        .ok_or("schema is not a native error family")?;
    let schema = ErrorSchema {
        declaration: key.declaration,
        display_name: if enumeration.name.contains('.') {
            enumeration.name.clone()
        } else {
            format!("{owner}.{}", enumeration.name)
        },
        parameters: parameters
            .iter()
            .map(|parameter| {
                parameter.constraint.map(|constraint| match constraint {
                    TypeConstraint::Comparable => interface::TypeConstraint::Comparable,
                    TypeConstraint::Equatable => interface::TypeConstraint::Equatable,
                    TypeConstraint::Numeric => interface::TypeConstraint::Numeric,
                    TypeConstraint::Printable => interface::TypeConstraint::Printable,
                    TypeConstraint::Error => interface::TypeConstraint::Error,
                })
            })
            .collect(),
        variants: enumeration
            .variants
            .iter()
            .map(|variant| {
                Ok(ErrorSchemaVariant {
                    name: variant.name.clone(),
                    message: variant.message.clone(),
                    cause: variant
                        .fields
                        .iter()
                        .position(|(name, _)| name.eq_ignore_ascii_case("Cause"))
                        .map(u16::try_from)
                        .transpose()
                        .map_err(|_| "error Cause index overflow")?,
                    fields: variant
                        .fields
                        .iter()
                        .map(|(name, ty)| {
                            Ok(ErrorSchemaField {
                                name: name.clone(),
                                ty: semantic_type_identity(ty, owner, parameters, resolve)?,
                            })
                        })
                        .collect::<Result<_, String>>()?,
                })
            })
            .collect::<Result<_, String>>()?,
    };
    schema.validate().map_err(str::to_owned)?;
    Ok(schema)
}
