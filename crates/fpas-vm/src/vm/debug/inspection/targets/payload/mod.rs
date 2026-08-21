//! Shared runtime and metadata resolution for writable payload descendants.

use fpas_bytecode::{DebugType, DebugTypeId, EnumValue, Executable, Value};

use super::MutationPath;

/// Structured failure while resolving one payload child name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::vm::debug) enum PayloadError {
    /// The name is absent from the currently active payload.
    UnknownField {
        /// Active variant or wrapper label used in diagnostics.
        active: String,
    },
    /// The live value has no writable payload for this selector.
    UnsupportedActive {
        /// Active variant or wrapper label used in diagnostics.
        active: String,
    },
    /// Portable metadata is missing or inconsistent with the live value.
    UnavailableMetadata {
        /// Bounded explanation of the metadata failure.
        detail: String,
    },
}

/// Resolves one payload field name to a guarded path component and declared type.
pub(in crate::vm::debug) fn resolve(
    executable: &Executable,
    expected: DebugTypeId,
    current: &Value,
    name: &str,
) -> Result<(MutationPath, DebugTypeId), PayloadError> {
    match current {
        Value::Enum(enumeration) => enum_field(executable, expected, enumeration.body(), name),
        Value::ResultOk(_) => wrapper_field(
            executable,
            expected,
            name,
            WrapperKind::ResultOk,
            MutationPath::ResultOk,
        ),
        Value::ResultError(_) => wrapper_field(
            executable,
            expected,
            name,
            WrapperKind::ResultError,
            MutationPath::ResultError,
        ),
        Value::OptionSome(_) => wrapper_field(
            executable,
            expected,
            name,
            WrapperKind::OptionSome,
            MutationPath::OptionSome,
        ),
        Value::OptionNone => Err(PayloadError::UnsupportedActive {
            active: active_label(current),
        }),
        _ => Err(PayloadError::UnsupportedActive {
            active: active_label(current),
        }),
    }
}

/// Human-readable label for the currently active payload-bearing value.
pub(in crate::vm::debug) fn active_label(value: &Value) -> String {
    match value {
        Value::Enum(enumeration) => {
            let layout = &enumeration.body().layout;
            format!("{}.{}", layout.type_name, layout.variant)
        }
        other => other.type_name().to_string(),
    }
}

#[derive(Clone, Copy)]
enum WrapperKind {
    ResultOk,
    ResultError,
    OptionSome,
}

fn enum_field(
    executable: &Executable,
    expected: DebugTypeId,
    body: &EnumValue,
    name: &str,
) -> Result<(MutationPath, DebugTypeId), PayloadError> {
    let active = format!("{}.{}", body.layout.type_name, body.layout.variant);
    let Some(index) = body
        .layout
        .fields
        .iter()
        .position(|field| field.eq_ignore_ascii_case(name))
    else {
        return Err(PayloadError::UnknownField { active });
    };
    let Some(DebugType::Enum(enumeration)) = debug_type(executable, expected) else {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("declared type for `{active}` is not an enum layout"),
        });
    };
    if body.layout.enumeration != *enumeration {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("enum `{active}` does not match its declared type owner"),
        });
    }
    let Some(variant) = executable
        .enum_variants
        .get(usize::from(body.layout.variant_id.get()))
    else {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("enum variant metadata for `{active}` is unavailable"),
        });
    };
    if variant.owner != body.layout.enumeration || variant.owner != *enumeration {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("enum variant `{active}` is not owned by its declared type"),
        });
    }
    if body.values.len() != variant.field_types.len() {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("enum variant `{active}` field count does not match metadata"),
        });
    }
    let Some(field_type) = variant.field_types.get(index).copied() else {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("enum field `{name}` on `{active}` is out of range"),
        });
    };
    require_declared_type(executable, field_type)?;
    Ok((
        MutationPath::EnumField {
            variant: body.layout.variant_id,
            index,
        },
        field_type,
    ))
}

fn wrapper_field(
    executable: &Executable,
    expected: DebugTypeId,
    name: &str,
    kind: WrapperKind,
    component: MutationPath,
) -> Result<(MutationPath, DebugTypeId), PayloadError> {
    let active = match kind {
        WrapperKind::ResultOk => "Result.Ok",
        WrapperKind::ResultError => "Result.Error",
        WrapperKind::OptionSome => "Option.Some",
    };
    if !name.eq_ignore_ascii_case("value") {
        return Err(PayloadError::UnknownField {
            active: active.to_string(),
        });
    }
    let declared = match (kind, debug_type(executable, expected)) {
        (WrapperKind::ResultOk, Some(DebugType::Result { ok, .. })) => *ok,
        (WrapperKind::ResultError, Some(DebugType::Result { error, .. })) => *error,
        (WrapperKind::OptionSome, Some(DebugType::Option(inner))) => *inner,
        (kind, Some(_)) => {
            return Err(PayloadError::UnavailableMetadata {
                detail: format!(
                    "declared type for `{active}` does not match the live {} payload",
                    match kind {
                        WrapperKind::ResultOk => "Result.Ok",
                        WrapperKind::ResultError => "Result.Error",
                        WrapperKind::OptionSome => "Option.Some",
                    }
                ),
            });
        }
        (_, None) => {
            return Err(PayloadError::UnavailableMetadata {
                detail: format!("declared type for `{active}` is unavailable"),
            });
        }
    };
    require_declared_type(executable, declared)?;
    Ok((component, declared))
}

fn debug_type(executable: &Executable, expected: DebugTypeId) -> Option<&DebugType> {
    executable.debug_types.get(expected.get() as usize)
}

fn require_declared_type(executable: &Executable, ty: DebugTypeId) -> Result<(), PayloadError> {
    if debug_type(executable, ty).is_some() {
        Ok(())
    } else {
        Err(PayloadError::UnavailableMetadata {
            detail: format!("payload field type #{} is unavailable", ty.get()),
        })
    }
}

#[cfg(test)]
mod tests;
