//! Copy-on-write rebuilding of supported writable aggregate paths.

use fpas_bytecode::Value;

use super::super::inspection::{MutationPath, active_label};
use super::super::types::{DebugErrorKind, DebugSessionError};

pub(in crate::vm::debug) fn descendant(
    mut root: Value,
    path: &[MutationPath],
    replacement: Value,
) -> Result<Value, DebugSessionError> {
    replace_at(&mut root, path, replacement)?;
    Ok(root)
}

pub(in crate::vm::debug) fn resolve<'a>(
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

    use fpas_bytecode::{EnumTypeId, EnumVariantId, RuntimeEnumLayout, SharedEnum, Value};

    use super::descendant;
    use crate::vm::debug::inspection::MutationPath;

    fn choice(variant_id: u16, variant: &str, values: Vec<Value>) -> Value {
        let fields = if values.len() == 2 {
            vec!["Left".to_string(), "Right".to_string()]
        } else {
            vec!["Value".to_string()]
        };
        Value::Enum(SharedEnum::new(
            Arc::new(RuntimeEnumLayout {
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
            crate::vm::debug::types::DebugErrorKind::VariableUnavailable
        );

        let ok = Value::ResultOk(Box::new(Value::Integer(1)));
        let replaced = descendant(ok.clone(), &[MutationPath::ResultOk], Value::Integer(4))
            .expect("ok payload");
        assert_eq!(replaced, Value::ResultOk(Box::new(Value::Integer(4))));
        assert_eq!(ok, Value::ResultOk(Box::new(Value::Integer(1))));

        let branch = descendant(
            Value::ResultError(Box::new(Value::Str("old".into()))),
            &[MutationPath::ResultOk],
            Value::Integer(4),
        )
        .expect_err("result branch guard");
        assert_eq!(
            branch.kind,
            crate::vm::debug::types::DebugErrorKind::VariableUnavailable
        );

        let none = descendant(
            Value::OptionNone,
            &[MutationPath::OptionSome],
            Value::Integer(4),
        )
        .expect_err("none has no payload");
        assert_eq!(
            none.kind,
            crate::vm::debug::types::DebugErrorKind::VariableUnavailable
        );
    }
}
