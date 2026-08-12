//! Detached construction of verified data-enum values.

use std::sync::Arc;

use fpas_bytecode::{RuntimeEnumLayout, SharedEnum, Value, VerifiedExecutable};

use super::detach::{ValueDetacher, error};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

/// Build a detached enum value from verified variant metadata and constructor arguments.
pub(super) fn construct(
    executable: &VerifiedExecutable,
    layout: Arc<RuntimeEnumLayout>,
    arguments: Vec<Value>,
    detacher: &mut ValueDetacher,
    max_depth: usize,
) -> Result<Value, DebugSessionError> {
    let image = executable.executable();
    let variant = image
        .enum_variants
        .get(layout.variant_id.get() as usize)
        .ok_or_else(|| {
            error(
                DebugErrorKind::UnknownCallable,
                format!(
                    "debug enum constructor `{}.{}` references a missing variant slot",
                    layout.type_name, layout.variant
                ),
                "Rebuild the executable with the current compiler.",
            )
        })?;
    if variant.owner != layout.enumeration {
        return Err(error(
            DebugErrorKind::UnknownCallable,
            format!(
                "debug enum constructor `{}.{}` has a mismatched owner in executable metadata",
                layout.type_name, layout.variant
            ),
            "Rebuild the executable with the current compiler.",
        ));
    }
    if variant.fields.len() != layout.fields.len()
        || variant.field_types.len() != layout.fields.len()
    {
        return Err(error(
            DebugErrorKind::UnknownCallable,
            format!(
                "debug enum constructor `{}.{}` has malformed field metadata",
                layout.type_name, layout.variant
            ),
            "Rebuild the executable with the current compiler.",
        ));
    }
    for field_type in &variant.field_types {
        if image.debug_types.get(field_type.get() as usize).is_none() {
            return Err(error(
                DebugErrorKind::UnknownCallable,
                format!(
                    "debug enum constructor `{}.{}` references an unavailable field type",
                    layout.type_name, layout.variant
                ),
                "Rebuild the executable with the current compiler.",
            ));
        }
    }
    if arguments.len() != layout.fields.len() {
        return Err(error(
            DebugErrorKind::CallArity,
            format!(
                "debug enum constructor `{}.{}` expects {} arguments, received {}",
                layout.type_name,
                layout.variant,
                layout.fields.len(),
                arguments.len()
            ),
            "Pass every associated field in declaration order, or omit arguments for a fieldless variant.",
        ));
    }
    for (index, (field_type, argument)) in variant.field_types.iter().zip(&arguments).enumerate() {
        crate::vm::debug::mutation::validate_value(
            executable,
            *field_type,
            argument,
            max_depth,
        )
        .map_err(|validation| {
            error(
                DebugErrorKind::EvaluationType,
                format!(
                    "debug enum constructor `{}.{}` argument {} for field `{}` does not match type #{}: {}",
                    layout.type_name,
                    layout.variant,
                    index + 1,
                    layout.fields[index],
                    field_type.get(),
                    validation.message
                ),
                "Pass an expression whose complete value matches the declared enum field type.",
            )
        })?;
    }
    let values = arguments
        .iter()
        .map(|value| detacher.detach(value))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Enum(SharedEnum::new(layout, values)))
}
