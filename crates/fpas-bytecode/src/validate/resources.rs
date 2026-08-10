//! Central executable resource-limit validation.

use crate::limits;

use super::{ValidationError, ValidationErrorKind, limit};

pub(super) fn validate_resources(executable: &crate::Executable) -> Result<(), ValidationError> {
    limit(
        "instructions",
        executable.code.len(),
        limits::MAX_INSTRUCTIONS,
    )?;
    limit(
        "functions",
        executable.functions.len(),
        limits::MAX_FUNCTIONS,
    )?;
    limit("strings", executable.strings.len(), limits::MAX_STRINGS)?;
    limit(
        "constants",
        executable.constants.len(),
        limits::MAX_CONSTANTS,
    )?;
    limit("globals", executable.globals.len(), limits::MAX_GLOBALS)?;
    limit(
        "record layouts",
        executable.records.len(),
        limits::MAX_RECORD_LAYOUTS,
    )?;
    limit(
        "enum layouts",
        executable.enums.len(),
        limits::MAX_ENUM_LAYOUTS,
    )?;
    limit(
        "enum variants",
        executable.enum_variants.len(),
        limits::MAX_ENUM_VARIANTS,
    )?;
    limit(
        "source paths",
        executable.source_map.sources.len(),
        limits::MAX_SOURCE_PATHS,
    )?;
    limit(
        "source runs",
        executable.source_map.runs.len(),
        limits::MAX_SOURCE_RUNS,
    )?;
    let debug_scopes = executable
        .functions
        .iter()
        .map(|function| function.debug.scopes.len())
        .sum();
    limit("debug scopes", debug_scopes, limits::MAX_DEBUG_SCOPES)?;
    let debug_bindings = executable
        .functions
        .iter()
        .map(|function| function.debug.bindings.len())
        .sum();
    limit("debug bindings", debug_bindings, limits::MAX_DEBUG_BINDINGS)?;
    let debug_points = executable
        .functions
        .iter()
        .map(|function| function.debug.sequence_points.len())
        .sum();
    limit(
        "debug sequence points",
        debug_points,
        limits::MAX_DEBUG_SEQUENCE_POINTS,
    )?;

    let mut string_bytes = 0_usize;
    for value in executable.strings.iter() {
        string_bytes = string_bytes.checked_add(value.len()).ok_or_else(|| {
            ValidationError::executable(ValidationErrorKind::ResourceLimit {
                resource: "string bytes",
                actual: usize::MAX,
                maximum: limits::MAX_STRING_BYTES,
            })
        })?;
    }
    limit("string bytes", string_bytes, limits::MAX_STRING_BYTES)?;
    for record in &executable.records {
        limit(
            "record fields",
            record.fields.len(),
            limits::MAX_LAYOUT_FIELDS,
        )?;
        limit(
            "record properties",
            record.properties.len(),
            limits::MAX_LAYOUT_FIELDS,
        )?;
    }
    for variant in &executable.enum_variants {
        limit(
            "enum fields",
            variant.fields.len(),
            limits::MAX_LAYOUT_FIELDS,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_limit_check_rejects_one_past_the_maximum() {
        assert!(matches!(
            limit("test", 5, 4),
            Err(ValidationError {
                kind: ValidationErrorKind::ResourceLimit {
                    resource: "test",
                    actual: 5,
                    maximum: 4
                },
                ..
            })
        ));
    }
}
