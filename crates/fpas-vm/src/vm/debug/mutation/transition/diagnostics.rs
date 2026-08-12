//! Actionable errors for qualified variant-transition assignment.

use super::super::super::types::{DebugErrorKind, DebugSessionError};
use super::suffix::VariantInfo;

pub(super) fn unqualified_or_unknown(
    name: &str,
    type_name: &str,
    variants: &[VariantInfo],
) -> DebugSessionError {
    let owners = variants
        .iter()
        .filter(|variant| {
            variant
                .fields
                .iter()
                .any(|field| field.eq_ignore_ascii_case(name))
        })
        .collect::<Vec<_>>();
    if owners.is_empty() {
        return unknown_variant(name, type_name, variants);
    }
    let example = owners
        .first()
        .map(|variant| {
            qualified_example(
                &variant.name,
                variant
                    .fields
                    .iter()
                    .find(|field| field.eq_ignore_ascii_case(name))
                    .map(String::as_str),
            )
        })
        .unwrap_or_else(|| format!("{type_name} constructor"));
    let constructor = owners
        .first()
        .map(|variant| constructor_example(type_name, &variant.name, variant.fields.len()))
        .unwrap_or_else(|| format!("a complete `{type_name}` constructor"));
    unsupported_path(
        format!(
            "debug variable target field `{name}` is not a writable payload of an inactive `{type_name}` variant"
        ),
        format!(
            "Name the variant explicitly, such as `{example}`, or replace the complete binding with `{constructor}`."
        ),
    )
}

pub(super) fn unknown_variant(
    name: &str,
    type_name: &str,
    variants: &[VariantInfo],
) -> DebugSessionError {
    let example = variants
        .iter()
        .find(|variant| variant.fields.len() == 1)
        .map(|variant| qualified_example(&variant.name, variant.fields.first().map(String::as_str)))
        .or_else(|| {
            variants
                .first()
                .map(|variant| constructor_example(type_name, &variant.name, variant.fields.len()))
        })
        .unwrap_or_else(|| format!("{type_name} constructor"));
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetUnknown,
        message: format!("debug variable target variant `{name}` does not exist on `{type_name}`"),
        hint: format!("Use an exact variant name from executable metadata, such as `{example}`."),
    }
}

pub(super) fn unknown_payload(
    name: &str,
    variant: &str,
    field: &str,
    qualified: &str,
    constructor: &str,
) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetUnknown,
        message: format!(
            "debug variable target field `{name}` does not exist on variant `{variant}`; expected `{field}`"
        ),
        hint: format!(
            "End the target at `{qualified}`, or replace the complete binding with `{constructor}`."
        ),
    }
}

pub(super) fn fieldless_variant(variant: &str, constructor: &str) -> DebugSessionError {
    unsupported_path(
        format!(
            "debug variable target cannot construct fieldless variant `{variant}` through a descendant"
        ),
        format!("Replace the complete binding with `{constructor}`."),
    )
}

pub(super) fn multi_field_variant(variant: &str, constructor: &str) -> DebugSessionError {
    unsupported_path(
        format!(
            "debug variable target cannot construct multi-field variant `{variant}` one field at a time"
        ),
        format!("Replace the complete binding with `{constructor}`."),
    )
}

pub(super) fn incomplete_payload(
    variant: &str,
    field: &str,
    qualified: &str,
    constructor: &str,
) -> DebugSessionError {
    unsupported_path(
        format!("debug variable target variant `{variant}` must end at payload field `{field}`"),
        format!("Assign `{qualified}`, or replace the complete binding with `{constructor}`."),
    )
}

pub(super) fn nested_inactive_payload(qualified: &str, constructor: &str) -> DebugSessionError {
    unsupported_path(
        "debug variable target cannot assign nested descendants of an inactive variant payload",
        format!(
            "Assign `{qualified}` with the complete payload, or replace the complete binding with `{constructor}`."
        ),
    )
}

pub(super) fn constructor_example(type_name: &str, variant: &str, field_count: usize) -> String {
    match (type_name, variant, field_count) {
        ("Option", "None", _) | (_, "None", 0) => "None".to_string(),
        ("Option", "Some", _) => "Some(...)".to_string(),
        ("Result", "Ok", _) => "Ok(...)".to_string(),
        ("Result", "Error", _) => "Error(...)".to_string(),
        (_, _, 0) => format!("{type_name}.{variant}"),
        (_, _, _) => format!("{type_name}.{variant}(...)"),
    }
}

pub(super) fn qualified_example(variant: &str, field: Option<&str>) -> String {
    match field {
        Some(field) => format!("{variant}.{field}"),
        None => variant.to_string(),
    }
}

pub(super) fn unsupported_metadata() -> DebugSessionError {
    unsupported_path(
        "debug variable target lacks assignable portable type metadata",
        "Select a descendant already supported by setVariable, or replace the complete binding with a constructor.",
    )
}

fn unsupported_path(message: impl Into<String>, hint: impl Into<String>) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: message.into(),
        hint: hint.into(),
    }
}
