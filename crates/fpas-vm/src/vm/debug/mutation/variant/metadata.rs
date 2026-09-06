//! Normalized enum, `Result`, and `Option` descriptors from portable debug metadata.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::sync::Arc;

use fpas_bytecode::{
    DebugType, DebugTypeId, EnumTypeId, EnumVariantId, Executable, RuntimeEnumLayout,
};

use super::diagnostics::{not_a_wrapper, unsupported_metadata};
use super::model::{VariantFieldMetadata, VariantKind, VariantMetadata, WrapperMetadata};

/// Return wrapper metadata when `expected` is an enum, `Result`, or `Option`.
///
/// Returns `Ok(None)` when the declared type is a different construct, so callers
/// can keep their existing diagnostics.
pub(in crate::vm::debug) fn try_wrapper(
    executable: &Executable,
    expected: DebugTypeId,
) -> Result<Option<WrapperMetadata>, crate::vm::debug::types::DebugSessionError> {
    match debug_type(executable, expected) {
        Some(DebugType::Enum(enumeration)) => enum_wrapper(executable, *enumeration).map(Some),
        Some(DebugType::Result { ok, error }) => result_wrapper(executable, *ok, *error).map(Some),
        Some(DebugType::Option(inner)) => option_wrapper(executable, *inner).map(Some),
        Some(_) => Ok(None),
        None => Err(unsupported_metadata()),
    }
}

/// Require enum, `Result`, or `Option` metadata for explicit discovery and construction.
pub(in crate::vm::debug) fn require_wrapper(
    executable: &Executable,
    expected: DebugTypeId,
) -> Result<WrapperMetadata, crate::vm::debug::types::DebugSessionError> {
    match try_wrapper(executable, expected)? {
        Some(wrapper) => Ok(wrapper),
        None => {
            let type_name = format_debug_type(executable, expected)?;
            Err(not_a_wrapper(&type_name))
        }
    }
}

fn enum_wrapper(
    executable: &Executable,
    enumeration: EnumTypeId,
) -> Result<WrapperMetadata, crate::vm::debug::types::DebugSessionError> {
    let type_name = enum_type_name(executable, enumeration)?;
    let variants = enum_variants(executable, enumeration, &type_name)?;
    Ok(WrapperMetadata {
        type_name,
        variants,
    })
}

fn result_wrapper(
    executable: &Executable,
    ok: DebugTypeId,
    error: DebugTypeId,
) -> Result<WrapperMetadata, crate::vm::debug::types::DebugSessionError> {
    let ok_type = format_debug_type(executable, ok)?;
    let error_type = format_debug_type(executable, error)?;
    Ok(WrapperMetadata {
        type_name: "Result".to_string(),
        variants: vec![
            wrapper_variant("Ok", "value", ok, ok_type, VariantKind::ResultOk),
            wrapper_variant(
                "Error",
                "value",
                error,
                error_type,
                VariantKind::ResultError,
            ),
        ],
    })
}

fn option_wrapper(
    executable: &Executable,
    inner: DebugTypeId,
) -> Result<WrapperMetadata, crate::vm::debug::types::DebugSessionError> {
    let inner_type = format_debug_type(executable, inner)?;
    Ok(WrapperMetadata {
        type_name: "Option".to_string(),
        variants: vec![
            wrapper_variant("Some", "value", inner, inner_type, VariantKind::OptionSome),
            VariantMetadata {
                canonical_name: "None".to_string(),
                name: "None".to_string(),
                fields: Vec::new(),
                kind: VariantKind::OptionNone,
                variant_id: None,
            },
        ],
    })
}

fn wrapper_variant(
    name: &str,
    field: &str,
    ty: DebugTypeId,
    type_name: String,
    kind: VariantKind,
) -> VariantMetadata {
    VariantMetadata {
        canonical_name: name.to_string(),
        name: name.to_string(),
        fields: vec![VariantFieldMetadata {
            name: field.to_string(),
            ty,
            type_name,
        }],
        kind,
        variant_id: None,
    }
}

fn enum_variants(
    executable: &Executable,
    enumeration: EnumTypeId,
    type_name: &str,
) -> Result<Vec<VariantMetadata>, crate::vm::debug::types::DebugSessionError> {
    executable
        .enum_variants
        .iter()
        .enumerate()
        .filter(|(_, variant)| variant.owner == enumeration)
        .map(|(index, variant)| {
            let name = required_string(executable, variant.name)?;
            let fields = variant
                .fields
                .iter()
                .map(|field| required_string(executable, *field))
                .collect::<Result<Vec<_>, _>>()?;
            if variant.field_types.len() != fields.len() {
                return Err(unsupported_metadata());
            }
            let variant_id =
                EnumVariantId::try_from_index(index).map_err(|_| unsupported_metadata())?;
            let display_fields = variant
                .field_types
                .iter()
                .zip(&fields)
                .map(|(ty, field_name)| {
                    Ok(VariantFieldMetadata {
                        name: field_name.clone(),
                        ty: *ty,
                        type_name: format_debug_type(executable, *ty)?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let layout = Arc::new(RuntimeEnumLayout {
                enumeration,
                variant_id,
                type_name: type_name.to_string(),
                variant: name.clone(),
                fields: fields.clone(),
            });
            Ok(VariantMetadata {
                canonical_name: format!("{type_name}.{name}"),
                name,
                fields: display_fields,
                kind: VariantKind::Enum { layout },
                variant_id: Some(variant_id),
            })
        })
        .collect()
}

fn enum_type_name(
    executable: &Executable,
    enumeration: EnumTypeId,
) -> Result<String, crate::vm::debug::types::DebugSessionError> {
    let layout = executable
        .enums
        .get(usize::from(enumeration.get()))
        .ok_or_else(unsupported_metadata)?;
    required_string(executable, layout.name)
}

pub(super) fn format_debug_type(
    executable: &Executable,
    id: DebugTypeId,
) -> Result<String, crate::vm::debug::types::DebugSessionError> {
    match debug_type(executable, id) {
        Some(DebugType::Unit) => Ok("unit".to_string()),
        Some(DebugType::Boolean) => Ok("Boolean".to_string()),
        Some(DebugType::Integer) => Ok("Integer".to_string()),
        Some(DebugType::Real) => Ok("Real".to_string()),
        Some(DebugType::String) => Ok("String".to_string()),
        Some(DebugType::Dynamic) => Ok("Dynamic".to_string()),
        Some(DebugType::Array(inner)) => Ok(format!(
            "array of {}",
            format_debug_type(executable, *inner)?
        )),
        Some(DebugType::Dictionary { key, value }) => Ok(format!(
            "dict of {} to {}",
            format_debug_type(executable, *key)?,
            format_debug_type(executable, *value)?
        )),
        Some(DebugType::Result { ok, error }) => Ok(format!(
            "result of {}, {}",
            format_debug_type(executable, *ok)?,
            format_debug_type(executable, *error)?
        )),
        Some(DebugType::Option(inner)) => Ok(format!(
            "option of {}",
            format_debug_type(executable, *inner)?
        )),
        Some(DebugType::Function { .. }) => Ok("function".to_string()),
        Some(DebugType::Record(record)) => {
            let layout = executable
                .records
                .get(usize::from(record.get()))
                .ok_or_else(unsupported_metadata)?;
            required_string(executable, layout.name)
        }
        Some(DebugType::Enum(enumeration)) => enum_type_name(executable, *enumeration),
        Some(DebugType::Cell(inner)) => format_debug_type(executable, *inner),
        Some(DebugType::Task(inner)) => Ok(format!(
            "task of {}",
            format_debug_type(executable, *inner)?
        )),
        Some(DebugType::Channel(inner)) => Ok(format!(
            "channel of {}",
            format_debug_type(executable, *inner)?
        )),
        None => Err(unsupported_metadata()),
    }
}

fn required_string(
    executable: &Executable,
    id: fpas_bytecode::StringId,
) -> Result<String, crate::vm::debug::types::DebugSessionError> {
    executable
        .strings
        .get(id)
        .map(str::to_string)
        .ok_or_else(unsupported_metadata)
}

fn debug_type(executable: &Executable, expected: DebugTypeId) -> Option<&DebugType> {
    executable.debug_types.get(expected.get() as usize)
}
