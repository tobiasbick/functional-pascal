//! Admission of essential error identities, literal labels and typed causes.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use std::collections::HashSet;

use super::{ValidationError, ValidationErrorKind};
use crate::Executable;

pub(super) fn validate(executable: &Executable) -> Result<(), ValidationError> {
    let mut keys = HashSet::new();
    let mut has_variants = vec![false; executable.enums.len()];
    for family in &executable.enums {
        if let Some(key) = &family.error {
            key.validate(0).map_err(invalid)?;
            if !keys.insert(key) {
                return Err(invalid("duplicate concrete error instance"));
            }
            let name = executable
                .strings
                .get(family.name)
                .ok_or_else(|| invalid("missing error name"))?;
            if !name.eq_ignore_ascii_case(&key.declaration) {
                return Err(invalid("error key disagrees with its defining name"));
            }
        }
    }
    for variant in &executable.enum_variants {
        let family = executable
            .enums
            .get(usize::from(variant.owner.get()))
            .ok_or_else(|| invalid("missing error owner"))?;
        has_variants[usize::from(variant.owner.get())] = true;
        if family.error.is_some() != variant.error.is_some() {
            return Err(invalid(
                "error variant metadata disagrees with its family kind",
            ));
        }
        let Some(error) = &variant.error else {
            continue;
        };
        if error.field_types.len() != variant.fields.len() {
            return Err(invalid(
                "error semantic field types disagree with payload slots",
            ));
        }
        crate::TypeIdentity::validate_all(&error.field_types, 0).map_err(invalid)?;
        if error.field_types.iter().any(|ty| !ty.is_printable()) {
            return Err(invalid("error payload requires Printable"));
        }
        if let Some(message) = error.message
            && executable.strings.get(message).is_none_or(str::is_empty)
        {
            return Err(invalid("error message must reference a nonempty literal"));
        }
        let mut designated = None;
        for (index, name) in variant.fields.iter().enumerate() {
            if executable
                .strings
                .get(*name)
                .is_some_and(|name| name.eq_ignore_ascii_case("Cause"))
                && designated.replace(index).is_some()
            {
                return Err(invalid("duplicate Cause field"));
            }
        }
        if designated != error.cause.as_ref().map(|cause| usize::from(cause.field)) {
            return Err(invalid(
                "error Cause slot disagrees with its field declaration",
            ));
        }
        if let Some(cause) = &error.cause {
            let family = executable
                .enums
                .get(usize::from(cause.family.get()))
                .and_then(|family| family.error.as_ref())
                .ok_or_else(|| invalid("Cause family is not an error instance"))?;
            if error.field_types.get(usize::from(cause.field)) != Some(&family.as_type()) {
                return Err(invalid(
                    "error Cause must reference a concrete error family",
                ));
            }
        }
    }
    for (family, present) in executable.enums.iter().zip(has_variants) {
        if family.error.is_some() && !present {
            return Err(invalid("error family has no variants"));
        }
    }
    Ok(())
}

fn invalid(reason: &'static str) -> ValidationError {
    ValidationError::executable(ValidationErrorKind::ErrorMetadata { reason })
}
