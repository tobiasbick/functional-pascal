//! Copy-on-write rebuilding of supported writable aggregate paths.

use crate::Value;

use super::super::inspection::{MutationPath, active_label};
use super::super::types::{DebugErrorKind, DebugSessionError};

pub(in crate::execution::vm::debug) fn descendant(
    mut root: Value,
    path: &[MutationPath],
    replacement: Value,
) -> Result<Value, DebugSessionError> {
    replace_at(&mut root, path, replacement)?;
    Ok(root)
}

/// Reads a concretized descendant without changing its root; see `docs/pascal/tools/debugger.md`.
pub(in crate::execution::vm::debug) fn resolve<'a>(
    mut value: &'a Value,
    path: &[MutationPath],
) -> Option<&'a Value> {
    for component in path {
        value = match (component, value) {
            (MutationPath::RecordField(index), Value::Record(record)) => {
                record.body().values.get(*index)?
            }
            (MutationPath::ArrayIndex(index), Value::Array(array)) => array.get(*index)?,
            (MutationPath::DictionaryValue(key), Value::Dict(dictionary)) => dictionary
                .iter()
                .find(|(candidate, _)| candidate == key)
                .map(|(_, value)| value)?,
            (MutationPath::EnumField { variant, index }, Value::Enum(enumeration))
                if enumeration.body().layout.variant_id == *variant =>
            {
                enumeration.body().values.get(*index)?
            }
            (MutationPath::ResultOk, Value::ResultOk(inner)) => inner.as_ref(),
            (MutationPath::ResultError, Value::ResultError(inner)) => inner.as_ref(),
            (MutationPath::OptionSome, Value::OptionSome(inner)) => inner.as_ref(),
            _ => return None,
        };
    }
    Some(value)
}

fn replace_at(
    current: &mut Value,
    path: &[MutationPath],
    replacement: Value,
) -> Result<(), DebugSessionError> {
    let Some((component, rest)) = path.split_first() else {
        *current = replacement;
        return Ok(());
    };
    let child = match (component, current) {
        (MutationPath::RecordField(index), Value::Record(record)) => record
            .values_mut()
            .get_mut(*index)
            .ok_or_else(path_unavailable)?,
        (MutationPath::ArrayIndex(index), Value::Array(array)) => {
            array.get_mut(*index).ok_or_else(path_unavailable)?
        }
        (MutationPath::DictionaryValue(key), Value::Dict(dictionary)) => dictionary
            .iter_mut()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, value)| value)
            .ok_or_else(path_unavailable)?,
        (MutationPath::EnumField { variant, index }, Value::Enum(enumeration))
            if enumeration.body().layout.variant_id == *variant =>
        {
            if rest.is_empty()
                && let Some(cause) = enumeration
                    .body()
                    .layout
                    .error
                    .as_ref()
                    .and_then(|error| error.cause)
                && usize::from(cause.field) == *index
                && !matches!(&replacement, Value::Enum(value)
                        if value.body().layout.enumeration == cause.family && value.body().layout.error.is_some())
            {
                return Err(DebugSessionError {
                    kind: DebugErrorKind::EvaluationType,
                    message: "replacement Cause has a different concrete error family".to_owned(),
                    hint: "Preserve the declared typed cause when replacing an error payload."
                        .to_owned(),
                });
            }
            enumeration
                .values_mut()
                .get_mut(*index)
                .ok_or_else(path_unavailable)?
        }
        (MutationPath::ResultOk, Value::ResultOk(inner)) => inner.as_mut(),
        (MutationPath::ResultError, Value::ResultError(inner)) => inner.as_mut(),
        (MutationPath::OptionSome, Value::OptionSome(inner)) => inner.as_mut(),
        (component, live) => return Err(payload_unavailable(component, live)),
    };
    replace_at(child, rest, replacement)
}

fn path_unavailable() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableUnavailable,
        message: "debug variable aggregate path changed before commit".to_string(),
        hint: "Request the variable tree again and retry the update.".to_string(),
    }
}

fn payload_unavailable(component: &MutationPath, live: &Value) -> DebugSessionError {
    let expected = match component {
        MutationPath::EnumField { variant, .. } => {
            format!("enum variant #{}", variant.get())
        }
        MutationPath::ResultOk => "Result.Ok".to_string(),
        MutationPath::ResultError => "Result.Error".to_string(),
        MutationPath::OptionSome => "Option.Some".to_string(),
        MutationPath::RecordField(_) => "a record field".to_string(),
        MutationPath::ArrayIndex(_) => "an array element".to_string(),
        MutationPath::DictionaryValue(_) => "a dictionary value".to_string(),
    };
    DebugSessionError {
        kind: DebugErrorKind::VariableUnavailable,
        message: format!(
            "debug variable payload path expected {expected} but the live value is `{}`",
            active_label(live)
        ),
        hint: "Request the variable tree again and retry the update.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{EnumTypeId, EnumVariantId, RuntimeEnumLayout, SharedEnum, Value};

    use super::descendant;
    use crate::execution::vm::debug::inspection::MutationPath;

    #[test]
    fn changing_a_cause_requires_its_exact_family_even_without_debug_types() {
        use crate::{ErrorCause, RuntimeErrorVariant};
        let make_leaf = |family| {
            Value::Enum(
                SharedEnum::try_new(
                    Arc::new(RuntimeEnumLayout {
                        error: Some(RuntimeErrorVariant {
                            hide_payload: false,
                            message: None,
                            cause: None,
                        }),
                        enumeration: EnumTypeId::new(family),
                        variant_id: EnumVariantId::new(family),
                        type_name: "Example.Leaf".to_owned(),
                        variant: "Failed".to_owned(),
                        fields: vec![],
                    }),
                    vec![],
                )
                .expect("leaf"),
            )
        };
        let wrapper = Value::Enum(
            SharedEnum::try_new(
                Arc::new(RuntimeEnumLayout {
                    error: Some(RuntimeErrorVariant {
                        hide_payload: false,
                        message: None,
                        cause: Some(ErrorCause {
                            field: 0,
                            family: EnumTypeId::new(0),
                        }),
                    }),
                    enumeration: EnumTypeId::new(2),
                    variant_id: EnumVariantId::new(2),
                    type_name: "Example.Wrapper".to_owned(),
                    variant: "Failed".to_owned(),
                    fields: vec!["Cause".to_owned()],
                }),
                vec![make_leaf(0)],
            )
            .expect("wrapper"),
        );
        let path = [MutationPath::EnumField {
            variant: EnumVariantId::new(2),
            index: 0,
        }];
        let error =
            descendant(wrapper.clone(), &path, make_leaf(1)).expect_err("wrong Cause family");
        assert_eq!(error.kind, super::DebugErrorKind::EvaluationType);
        descendant(wrapper, &path, make_leaf(0)).expect("same-family replacement");
    }

    fn choice(variant_id: u16, variant: &str, values: Vec<Value>) -> Value {
        let fields = if values.len() == 2 {
            vec!["Left".to_string(), "Right".to_string()]
        } else {
            vec!["Value".to_string()]
        };
        Value::Enum(SharedEnum::new(
            Arc::new(RuntimeEnumLayout {
                error: None,
                enumeration: EnumTypeId::new(0),
                variant_id: EnumVariantId::new(variant_id),
                type_name: "Choice".to_string(),
                variant: variant.to_string(),
                fields,
            }),
            values,
        ))
    }

    #[test]
    fn payload_rebuild_is_copy_on_write_and_rejects_variant_drift() {
        let original = choice(0, "Count", vec![Value::Integer(1)]);
        let updated = descendant(
            original.clone(),
            &[MutationPath::EnumField {
                variant: EnumVariantId::new(0),
                index: 0,
            }],
            Value::Integer(9),
        )
        .expect("enum field replacement");
        assert_eq!(
            original,
            choice(0, "Count", vec![Value::Integer(1)]),
            "failed or sibling writes must not mutate the original root"
        );
        let Value::Enum(updated) = updated else {
            panic!("expected enum");
        };
        assert_eq!(updated.body().values[0], Value::Integer(9));

        let drifted = descendant(
            choice(1, "Pair", vec![Value::Integer(2), Value::Integer(3)]),
            &[MutationPath::EnumField {
                variant: EnumVariantId::new(0),
                index: 0,
            }],
            Value::Integer(9),
        )
        .expect_err("variant guard");
        assert_eq!(
            drifted.kind,
            crate::execution::vm::debug::types::DebugErrorKind::VariableUnavailable
        );

        let ok = Value::result_ok(Value::Integer(1));
        let replaced = descendant(ok.clone(), &[MutationPath::ResultOk], Value::Integer(4))
            .expect("ok payload");
        assert_eq!(replaced, Value::result_ok(Value::Integer(4)));
        assert_eq!(ok, Value::result_ok(Value::Integer(1)));

        let branch = descendant(
            Value::result_error(Value::Str("old".into())),
            &[MutationPath::ResultOk],
            Value::Integer(4),
        )
        .expect_err("result branch guard");
        assert_eq!(
            branch.kind,
            crate::execution::vm::debug::types::DebugErrorKind::VariableUnavailable
        );

        let none = descendant(
            Value::OptionNone,
            &[MutationPath::OptionSome],
            Value::Integer(4),
        )
        .expect_err("none has no payload");
        assert_eq!(
            none.kind,
            crate::execution::vm::debug::types::DebugErrorKind::VariableUnavailable
        );
    }
}
