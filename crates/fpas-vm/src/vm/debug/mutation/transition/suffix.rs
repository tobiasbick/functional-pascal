//! Metadata lookup and exact variant-suffix matching.

use std::sync::Arc;

use fpas_bytecode::{
    DebugType, DebugTypeId, EnumTypeId, EnumVariantId, Executable, RuntimeEnumLayout, Value,
};

use super::super::super::inspection::MutationPath;
use super::super::super::types::DebugSessionError;
use super::super::model::DebugAssignmentSelector;
use super::diagnostics::{
    constructor_example, fieldless_variant, incomplete_payload, multi_field_variant,
    nested_inactive_payload, qualified_example, unknown_payload, unknown_variant,
    unqualified_or_unknown, unsupported_metadata,
};
use super::{QualifiedSuffix, SuffixResolution, TransitionKind, TransitionSpec};

pub(super) fn resolve_enum_suffix(
    executable: &Executable,
    current: &Value,
    expected: DebugTypeId,
    remaining: &[DebugAssignmentSelector],
) -> Result<SuffixResolution, DebugSessionError> {
    let Some(DebugType::Enum(enumeration)) = debug_type(executable, expected) else {
        return Err(unsupported_metadata());
    };
    let type_name = enum_type_name(executable, *enumeration)?;
    let variants = enum_variants(executable, *enumeration)?;
    let Some(variant_name) = first_field(remaining) else {
        return Err(unknown_variant(
            first_selector_label(remaining),
            &type_name,
            &variants,
        ));
    };
    let matches = variants
        .iter()
        .filter(|variant| variant.name.eq_ignore_ascii_case(variant_name))
        .collect::<Vec<_>>();
    let variant = match matches.as_slice() {
        [variant] => *variant,
        [] => {
            return Ok(SuffixResolution::Unmatched(unqualified_or_unknown(
                variant_name,
                &type_name,
                &variants,
            )));
        }
        _ => {
            return Err(unknown_variant(variant_name, &type_name, &variants));
        }
    };
    finish_variant(
        remaining,
        variant,
        current_enum_is_active(current, variant),
        constructor_example(&type_name, &variant.name, variant.fields.len()),
        qualified_example(&variant.name, variant.fields.first().map(String::as_str)),
    )
    .map(SuffixResolution::Exact)
}

pub(super) fn resolve_result_suffix(
    executable: &Executable,
    current: &Value,
    expected: DebugTypeId,
    remaining: &[DebugAssignmentSelector],
) -> Result<SuffixResolution, DebugSessionError> {
    let Some(DebugType::Result { ok, error }) = debug_type(executable, expected) else {
        return Err(unsupported_metadata());
    };
    let ok_variant = wrapper_variant("Ok", "value", *ok, TransitionKind::ResultOk);
    let error_variant = wrapper_variant("Error", "value", *error, TransitionKind::ResultError);
    let variants = [ok_variant.clone(), error_variant.clone()];
    let Some(variant_name) = first_field(remaining) else {
        return Err(unknown_variant(
            first_selector_label(remaining),
            "Result",
            &variants,
        ));
    };
    let variant = if variant_name.eq_ignore_ascii_case("Ok") {
        &ok_variant
    } else if variant_name.eq_ignore_ascii_case("Error") {
        &error_variant
    } else {
        return Ok(SuffixResolution::Unmatched(unqualified_or_unknown(
            variant_name,
            "Result",
            &variants,
        )));
    };
    let active = matches!(
        (current, variant.name.as_str()),
        (Value::ResultOk(_), "Ok") | (Value::ResultError(_), "Error")
    );
    finish_variant(
        remaining,
        variant,
        active,
        constructor_example("Result", &variant.name, 1),
        qualified_example(&variant.name, Some("value")),
    )
    .map(SuffixResolution::Exact)
}

pub(super) fn resolve_option_suffix(
    executable: &Executable,
    current: &Value,
    expected: DebugTypeId,
    remaining: &[DebugAssignmentSelector],
) -> Result<SuffixResolution, DebugSessionError> {
    let Some(DebugType::Option(inner)) = debug_type(executable, expected) else {
        return Err(unsupported_metadata());
    };
    let some = wrapper_variant("Some", "value", *inner, TransitionKind::OptionSome);
    let none = VariantInfo {
        name: "None".to_string(),
        fields: Vec::new(),
        payload_type: None,
        kind: None,
        variant_id: None,
    };
    let variants = [some.clone(), none];
    let Some(variant_name) = first_field(remaining) else {
        return Err(unknown_variant(
            first_selector_label(remaining),
            "Option",
            &variants,
        ));
    };
    let variant = if variant_name.eq_ignore_ascii_case("Some") {
        &some
    } else if variant_name.eq_ignore_ascii_case("None") {
        &variants[1]
    } else {
        return Ok(SuffixResolution::Unmatched(unqualified_or_unknown(
            variant_name,
            "Option",
            &variants,
        )));
    };
    let active = matches!(
        (current, variant.name.as_str()),
        (Value::OptionSome(_), "Some") | (Value::OptionNone, "None")
    );
    finish_variant(
        remaining,
        variant,
        active,
        constructor_example("Option", &variant.name, variant.fields.len()),
        qualified_example(&variant.name, variant.fields.first().map(String::as_str)),
    )
    .map(SuffixResolution::Exact)
}

fn finish_variant(
    remaining: &[DebugAssignmentSelector],
    variant: &VariantInfo,
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
                    field,
                    &qualified,
                    &constructor,
                ));
            };
            if !payload_name.eq_ignore_ascii_case(field) {
                return Err(unknown_payload(
                    payload_name,
                    &variant.name,
                    field,
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
                    payload_type: variant.payload_type.ok_or_else(unsupported_metadata)?,
                    kind: variant.kind.clone().ok_or_else(unsupported_metadata)?,
                }))
            }
        }
        _ => Err(multi_field_variant(&variant.name, &constructor)),
    }
}

fn active_component(
    variant: &VariantInfo,
) -> Result<(MutationPath, DebugTypeId), DebugSessionError> {
    let expected = variant.payload_type.ok_or_else(unsupported_metadata)?;
    let component = match &variant.kind {
        Some(TransitionKind::ResultOk) => MutationPath::ResultOk,
        Some(TransitionKind::ResultError) => MutationPath::ResultError,
        Some(TransitionKind::OptionSome) => MutationPath::OptionSome,
        Some(TransitionKind::Enum { .. }) => MutationPath::EnumField {
            variant: variant.variant_id.ok_or_else(unsupported_metadata)?,
            index: 0,
        },
        None => return Err(unsupported_metadata()),
    };
    Ok((component, expected))
}

fn current_enum_is_active(current: &Value, variant: &VariantInfo) -> bool {
    match (current, variant.variant_id) {
        (Value::Enum(enumeration), Some(variant_id)) => {
            enumeration.body().layout.variant_id == variant_id
        }
        _ => false,
    }
}

#[derive(Clone)]
pub(super) struct VariantInfo {
    pub name: String,
    pub fields: Vec<String>,
    payload_type: Option<DebugTypeId>,
    kind: Option<TransitionKind>,
    variant_id: Option<EnumVariantId>,
}

fn wrapper_variant(
    name: &str,
    field: &str,
    payload_type: DebugTypeId,
    kind: TransitionKind,
) -> VariantInfo {
    VariantInfo {
        name: name.to_string(),
        fields: vec![field.to_string()],
        payload_type: Some(payload_type),
        kind: Some(kind),
        variant_id: None,
    }
}

fn enum_variants(
    executable: &Executable,
    enumeration: EnumTypeId,
) -> Result<Vec<VariantInfo>, DebugSessionError> {
    let type_name = enum_type_name(executable, enumeration)?;
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
            let layout = Arc::new(RuntimeEnumLayout {
                enumeration,
                variant_id,
                type_name: type_name.clone(),
                variant: name.clone(),
                fields: fields.clone(),
            });
            let (payload_type, kind) = match variant.field_types.as_slice() {
                [payload] => (
                    Some(*payload),
                    Some(TransitionKind::Enum {
                        layout: Arc::clone(&layout),
                    }),
                ),
                _ => (None, None),
            };
            Ok(VariantInfo {
                name,
                fields,
                payload_type,
                kind,
                variant_id: Some(variant_id),
            })
        })
        .collect()
}

fn enum_type_name(
    executable: &Executable,
    enumeration: EnumTypeId,
) -> Result<String, DebugSessionError> {
    let layout = executable
        .enums
        .get(usize::from(enumeration.get()))
        .ok_or_else(unsupported_metadata)?;
    required_string(executable, layout.name)
}

fn required_string(
    executable: &Executable,
    id: fpas_bytecode::StringId,
) -> Result<String, DebugSessionError> {
    executable
        .strings
        .get(id)
        .map(str::to_string)
        .ok_or_else(unsupported_metadata)
}

fn debug_type(executable: &Executable, expected: DebugTypeId) -> Option<&DebugType> {
    executable.debug_types.get(expected.get() as usize)
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
