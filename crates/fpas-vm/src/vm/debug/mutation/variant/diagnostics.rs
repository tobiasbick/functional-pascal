//! Actionable diagnostics for variant discovery and complete construction.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::model::{VariantMetadata, WrapperMetadata};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

pub(in crate::vm::debug) fn not_a_wrapper(type_name: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: format!(
            "debug variant target type `{type_name}` is not an enum, Result, or Option"
        ),
        hint: "Select a mutable enum, `result of T, E`, or `option of T` target such as `Selected` or `Outcome`."
            .to_string(),
    }
}

pub(in crate::vm::debug) fn unknown_variant(
    name: &str,
    wrapper: &WrapperMetadata,
) -> DebugSessionError {
    let example = wrapper
        .variants
        .first()
        .map(|variant| variant.canonical_name.as_str())
        .unwrap_or("Choice.Empty");
    DebugSessionError {
        kind: DebugErrorKind::VariantUnknown,
        message: format!(
            "debug variant `{name}` does not exist on `{type_name}`",
            type_name = wrapper.type_name
        ),
        hint: format!("Use a canonical variant name from `variant.describe`, such as `{example}`."),
    }
}

pub(in crate::vm::debug) fn missing_fields(
    variant: &VariantMetadata,
    missing: &[&str],
) -> DebugSessionError {
    let listed = join_names(missing);
    DebugSessionError {
        kind: DebugErrorKind::VariantFieldSet,
        message: format!(
            "debug variant `{canonical}` is missing field expressions for {listed}",
            canonical = variant.canonical_name
        ),
        hint: format!(
            "Supply exactly one expression for every declared field; for `{canonical}` that is {expected}.",
            canonical = variant.canonical_name,
            expected = expected_fields(variant)
        ),
    }
}

pub(in crate::vm::debug) fn extra_fields(
    variant: &VariantMetadata,
    extra: &[&str],
) -> DebugSessionError {
    let listed = join_names(extra);
    DebugSessionError {
        kind: DebugErrorKind::VariantFieldSet,
        message: format!(
            "debug variant `{canonical}` received unknown field {listed}",
            canonical = variant.canonical_name
        ),
        hint: format!(
            "Omit extra fields; `{canonical}` accepts {expected}.",
            canonical = variant.canonical_name,
            expected = expected_fields(variant)
        ),
    }
}

pub(in crate::vm::debug) fn duplicate_field(
    variant: &VariantMetadata,
    name: &str,
) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariantFieldSet,
        message: format!(
            "debug variant `{canonical}` received duplicate field `{name}`",
            canonical = variant.canonical_name
        ),
        hint: "Use each declared field name once; matching is ASCII-case-insensitive.".to_string(),
    }
}

pub(in crate::vm::debug) fn identity_bearing_field(
    variant: &VariantMetadata,
    field: &str,
) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!(
            "debug variant `{canonical}` cannot construct identity-bearing field `{field}`",
            canonical = variant.canonical_name
        ),
        hint: "Supply only ordinary values; function, task, and capture-cell fields remain outside this operation."
            .to_string(),
    }
}

pub(in crate::vm::debug) fn unsupported_metadata() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: "debug variant target lacks assignable portable type metadata".to_string(),
        hint: "Select a descendant already supported by setVariable, or replace the complete binding with a constructor."
            .to_string(),
    }
}

pub(in crate::vm::debug) fn constructor_example(
    type_name: &str,
    variant: &str,
    field_count: usize,
) -> String {
    match (type_name, variant, field_count) {
        ("Option", "None", _) | (_, "None", 0) => "None".to_string(),
        ("Option", "Some", _) => "Some(...)".to_string(),
        ("Result", "Ok", _) => "Ok(...)".to_string(),
        ("Result", "Error", _) => "Error(...)".to_string(),
        (_, _, 0) => format!("{type_name}.{variant}"),
        (_, _, _) => format!("{type_name}.{variant}(...)"),
    }
}

pub(in crate::vm::debug) fn qualified_example(variant: &str, field: Option<&str>) -> String {
    match field {
        Some(field) => format!("{variant}.{field}"),
        None => variant.to_string(),
    }
}

fn expected_fields(variant: &VariantMetadata) -> String {
    if variant.fields.is_empty() {
        "an empty `fields` object".to_string()
    } else {
        join_names(&variant.field_names())
    }
}

fn join_names(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
