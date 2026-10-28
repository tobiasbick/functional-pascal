//! Actionable errors for qualified variant-transition assignment.

use super::super::super::types::{DebugErrorKind, DebugSessionError};
use super::super::variant::{WrapperMetadata, constructor_example, qualified_example};

pub(super) fn unqualified_or_unknown(name: &str, wrapper: &WrapperMetadata) -> DebugSessionError {
    let owners = wrapper
        .variants
        .iter()
        .filter(|variant| {
            variant
                .fields
                .iter()
                .any(|field| field.name.eq_ignore_ascii_case(name))
        })
        .collect::<Vec<_>>();
    if owners.is_empty() {
        return unknown_variant(name, wrapper);
    }
    let example = owners
        .first()
        .map(|variant| {
            qualified_example(
                &variant.name,
                variant
                    .fields
                    .iter()
                    .find(|field| field.name.eq_ignore_ascii_case(name))
                    .map(|field| field.name.as_str()),
            )
        })
        .unwrap_or_else(|| format!("{} constructor", wrapper.type_name));
    let constructor = owners
        .first()
        .map(|variant| constructor_example(&wrapper.type_name, &variant.name, variant.fields.len()))
        .unwrap_or_else(|| format!("a complete `{}` constructor", wrapper.type_name));
    unsupported_path(
        format!(
            "debug variable target field `{name}` is not a writable payload of an inactive `{}` variant",
            wrapper.type_name
        ),
        format!(
            "Name the variant explicitly, such as `{example}`, or replace the complete binding with `{constructor}`."
        ),
    )
}

pub(super) fn unknown_variant(name: &str, wrapper: &WrapperMetadata) -> DebugSessionError {
    let example = wrapper
        .variants
        .iter()
        .find(|variant| variant.fields.len() == 1)
        .map(|variant| {
            qualified_example(
                &variant.name,
                variant.fields.first().map(|field| field.name.as_str()),
            )
        })
        .or_else(|| {
            wrapper.variants.first().map(|variant| {
                constructor_example(&wrapper.type_name, &variant.name, variant.fields.len())
            })
        })
        .unwrap_or_else(|| format!("{} constructor", wrapper.type_name));
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetUnknown,
        message: format!(
            "debug variable target variant `{name}` does not exist on `{}`",
            wrapper.type_name
        ),
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

fn unsupported_path(message: impl Into<String>, hint: impl Into<String>) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: message.into(),
        hint: hint.into(),
    }
}
