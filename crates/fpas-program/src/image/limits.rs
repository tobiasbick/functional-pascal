//! Program-image identity resource accounting.

use crate::{ImageError, ProgramIdentity};

pub(super) fn validate_identity_resources(identity: &ProgramIdentity) -> Result<(), ImageError> {
    check(
        "compiler_version",
        identity.compiler_version.len(),
        fpas_bytecode::limits::MAX_IDENTITY_STRING_BYTES,
    )?;
    check(
        "linked_units",
        identity.units.len(),
        fpas_bytecode::limits::MAX_LINKED_UNITS,
    )?;
    for unit in &identity.units {
        check(
            "unit_name",
            unit.unit_name.len(),
            fpas_bytecode::limits::MAX_IDENTITY_STRING_BYTES,
        )?;
    }
    Ok(())
}

fn check(field: &'static str, size: usize, maximum: usize) -> Result<(), ImageError> {
    if size > maximum {
        return Err(ImageError::ResourceLimit {
            field,
            size,
            maximum,
        });
    }
    Ok(())
}
