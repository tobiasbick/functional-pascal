//! Portable error labels and exact nominal causes in independently compiled objects.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::{ObjectError, RelocatableObject};

/// Retained declaration with its real object-local source provenance.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectErrorSchema {
    /// Semantic declaration, also usable in compiled interfaces.
    pub schema: crate::errors::ErrorSchema,
    /// Declaring source position for shared compiler-generated constructor bodies.
    pub location: super::ObjectDebugLocation,
}

/// Error-specific description preserved independently of optional debug types.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectErrorVariant {
    /// Concrete semantic field types, independent of optional debugger metadata.
    #[serde(deserialize_with = "fpas_code::TypeIdentity::deserialize_list")]
    pub field_types: Vec<fpas_code::TypeIdentity>,
    /// Optional nonempty literal label.
    pub message: Option<String>,
    /// Original typed cause, absent on leaf variants.
    pub cause: Option<u16>,
}

pub(super) fn validate(object: &RelocatableObject) -> Result<(), ObjectError> {
    let invalid = ObjectError::InvalidTableReference;
    crate::equality::EqualityDeclarations::new(object.equality_declarations.clone())
        .map_err(invalid)?;
    let mut declarations = std::collections::BTreeSet::new();
    for retained in &object.error_schemas {
        let schema = &retained.schema;
        schema.validate().map_err(invalid)?;
        if retained.location.source as usize >= object.sources.len()
            || retained.location.line == 0
            || retained.location.column == 0
        {
            return Err(invalid("error declaration source provenance"));
        }
        if !declarations.insert(&schema.declaration) {
            return Err(invalid("duplicate error schema"));
        }
    }
    for function in &object.functions {
        if let Some(signature) = &function.error_constructors {
            signature.validate().map_err(invalid)?;
            if signature.requirements.len() > usize::from(function.arity) {
                return Err(invalid("hidden error constructors exceed function arity"));
            }
        }
    }
    for layout in &object.enums {
        if let Some(key) = &layout.error {
            key.validate(0).map_err(invalid)?;
            if !layout.name.eq_ignore_ascii_case(&key.declaration) || layout.variants.is_empty() {
                return Err(invalid("error layout identity or variants"));
            }
        }
        for variant in &layout.variants {
            if layout.error.is_some() != variant.error.is_some() {
                return Err(invalid("ordinary/error variant metadata mismatch"));
            }
            let Some(error) = &variant.error else {
                continue;
            };
            if error.field_types.len() != variant.fields.len() {
                return Err(invalid(
                    "error semantic field types disagree with payload slots",
                ));
            }
            fpas_code::TypeIdentity::validate_all(&error.field_types, 0).map_err(invalid)?;
            if error.field_types.iter().any(|ty| !ty.is_printable()) {
                return Err(invalid("error payload requires Printable"));
            }
            if error.message.as_ref().is_some_and(String::is_empty) {
                return Err(invalid("empty error message"));
            }
            let causes = variant
                .fields
                .iter()
                .enumerate()
                .filter(|(_, name)| name.eq_ignore_ascii_case("Cause"))
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            if causes.len() > 1 || causes.first().copied() != error.cause.map(usize::from) {
                return Err(invalid("error Cause field and slot mismatch"));
            }
            if let Some(cause) = error.cause
                && error.field_types[usize::from(cause)].error_key().is_none()
            {
                return Err(invalid("Cause field is not an error type"));
            }
        }
    }
    for constant in &object.constants {
        if let super::ObjectConstant::ErrorConstructor(request) = constant {
            request.instance.validate(0).map_err(invalid)?;
        }
    }
    Ok(())
}
