//! Metadata lookup and exact variant-suffix matching.

use super::super::super::inspection::MutationPath;
use super::super::super::types::DebugSessionError;
use super::super::model::DebugAssignmentSelector;
use super::super::variant::{
    VariantKind, VariantMetadata, constructor_example, qualified_example, try_wrapper,
    unsupported_metadata,
};
use super::diagnostics::{
    fieldless_variant, incomplete_payload, multi_field_variant, nested_inactive_payload,
    unknown_payload, unknown_variant, unqualified_or_unknown,
};
use super::{QualifiedSuffix, SuffixResolution, TransitionKind, TransitionSpec};
use fpas_bytecode::{DebugTypeId, Executable, Value};

pub(super) fn resolve_enum_suffix(
    executable: &Executable,
    current: &Value,
    expected: DebugTypeId,
    remaining: &[DebugAssignmentSelector],
) -> Result<SuffixResolution, DebugSessionError> {
    resolve_wrapper_suffix(executable, current, expected, remaining)
}

pub(super) fn resolve_result_suffix(
    executable: &Executable,
    current: &Value,
    expected: DebugTypeId,
    remaining: &[DebugAssignmentSelector],
) -> Result<SuffixResolution, DebugSessionError> {
    resolve_wrapper_suffix(executable, current, expected, remaining)
}

pub(super) fn resolve_option_suffix(
    executable: &Executable,
    current: &Value,
    expected: DebugTypeId,
    remaining: &[DebugAssignmentSelector],
) -> Result<SuffixResolution, DebugSessionError> {
    resolve_wrapper_suffix(executable, current, expected, remaining)
}

fn resolve_wrapper_suffix(
    executable: &Executable,
    current: &Value,
    expected: DebugTypeId,
    remaining: &[DebugAssignmentSelector],
) -> Result<SuffixResolution, DebugSessionError> {
    let wrapper = try_wrapper(executable, expected)?.ok_or_else(unsupported_metadata)?;
    let Some(variant_name) = first_field(remaining) else {
        return Err(unknown_variant(first_selector_label(remaining), &wrapper));
    };
    let variant = match wrapper.find_short(variant_name) {
        Ok(variant) => variant,
        Err(matches) if matches.is_empty() => {
            return Ok(SuffixResolution::Unmatched(unqualified_or_unknown(
                variant_name,
                &wrapper,
            )));
        }
        Err(_) => {
            return Err(unknown_variant(variant_name, &wrapper));
        }
    };
    finish_variant(
        remaining,
        variant,
        current_is_active(current, variant),
        constructor_example(&wrapper.type_name, &variant.name, variant.fields.len()),
        qualified_example(
            &variant.name,
            variant.fields.first().map(|field| field.name.as_str()),
        ),
    )
    .map(SuffixResolution::Exact)
}

fn finish_variant(
    remaining: &[DebugAssignmentSelector],
    variant: &VariantMetadata,
    active: bool,
    constructor: String,
    qualified: String,
) -> Result<QualifiedSuffix, DebugSessionError> {
    match variant.fields.as_slice() {
        [] => Err(fieldless_variant(&variant.name, &constructor)),
        [field] => {
            let Some(payload_name) = second_field(remaining) else {
                return Err(incomplete_payload(
                    &variant.name,
                    &field.name,
                    &qualified,
                    &constructor,
                ));
            };
            if !payload_name.eq_ignore_ascii_case(&field.name) {
                return Err(unknown_payload(
                    payload_name,
                    &variant.name,
                    &field.name,
                    &qualified,
                    &constructor,
                ));
            }
            if remaining.len() != 2 {
                if active {
                    let (component, expected) = active_component(variant)?;
                    return Ok(QualifiedSuffix::ActivePayload {
                        component,
                        expected,
                        consumed: 2,
                    });
                }
                return Err(nested_inactive_payload(&qualified, &constructor));
            }
            if active {
                let (component, expected) = active_component(variant)?;
                Ok(QualifiedSuffix::ActivePayload {
                    component,
                    expected,
                    consumed: 2,
                })
            } else {
                Ok(QualifiedSuffix::Switch(TransitionSpec {
                    payload_type: variant.payload_type().ok_or_else(unsupported_metadata)?,
                    kind: transition_kind(variant)?,
                }))
            }
        }
        _ => Err(multi_field_variant(&variant.name, &constructor)),
    }
}

fn active_component(
    variant: &VariantMetadata,
) -> Result<(MutationPath, DebugTypeId), DebugSessionError> {
    let expected = variant.payload_type().ok_or_else(unsupported_metadata)?;
    let component = match &variant.kind {
        VariantKind::ResultOk => MutationPath::ResultOk,
        VariantKind::ResultError => MutationPath::ResultError,
        VariantKind::OptionSome => MutationPath::OptionSome,
        VariantKind::Enum { .. } => MutationPath::EnumField {
            variant: variant.variant_id.ok_or_else(unsupported_metadata)?,
            index: 0,
        },
        VariantKind::OptionNone => return Err(unsupported_metadata()),
    };
    Ok((component, expected))
}

fn transition_kind(variant: &VariantMetadata) -> Result<TransitionKind, DebugSessionError> {
    match &variant.kind {
        VariantKind::ResultOk => Ok(TransitionKind::ResultOk),
        VariantKind::ResultError => Ok(TransitionKind::ResultError),
        VariantKind::OptionSome => Ok(TransitionKind::OptionSome),
        VariantKind::Enum { layout } if variant.fields.len() == 1 => Ok(TransitionKind::Enum {
            layout: layout.clone(),
        }),
        VariantKind::Enum { .. } | VariantKind::OptionNone => Err(unsupported_metadata()),
    }
}

fn current_is_active(current: &Value, variant: &VariantMetadata) -> bool {
    match (current, &variant.kind, variant.variant_id) {
        (Value::Enum(enumeration), VariantKind::Enum { .. }, Some(variant_id)) => {
            enumeration.body().layout.variant_id == variant_id
        }
        (Value::ResultOk(_), VariantKind::ResultOk, _) => true,
        (Value::ResultError(_), VariantKind::ResultError, _) => true,
        (Value::OptionSome(_), VariantKind::OptionSome, _) => true,
        (Value::OptionNone, VariantKind::OptionNone, _) => true,
        _ => false,
    }
}

fn first_field(remaining: &[DebugAssignmentSelector]) -> Option<&str> {
    match remaining.first() {
        Some(DebugAssignmentSelector::Field(name)) => Some(name.as_str()),
        _ => None,
    }
}

fn second_field(remaining: &[DebugAssignmentSelector]) -> Option<&str> {
    match remaining.get(1) {
        Some(DebugAssignmentSelector::Field(name)) => Some(name.as_str()),
        _ => None,
    }
}

fn first_selector_label(remaining: &[DebugAssignmentSelector]) -> &str {
    first_field(remaining).unwrap_or("<index>")
}
