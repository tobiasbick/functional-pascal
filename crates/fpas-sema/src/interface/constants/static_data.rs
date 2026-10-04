//! Persistent aggregate data with the runtime's shared equality representation.

use std::sync::Arc;

use fpas_bytecode::{
    EnumTypeId, EnumVariantId, RecordTypeId, RuntimeEnumLayout, RuntimeRecordLayout, SharedEnum,
    SharedRecord, Value,
};
use fpas_unit::interface::StaticValue;

use super::conversion::{from_interface, from_scalar, to_interface, to_scalar};

/// Persist evaluated data; callable and resource identities are not static values.
pub(super) fn persist(value: &Value) -> Option<StaticValue> {
    Some(match value {
        Value::Array(values) => {
            StaticValue::Array(values.iter().map(persist).collect::<Option<_>>()?)
        }
        Value::Dict(pairs) => StaticValue::Dictionary(
            pairs
                .iter()
                .map(|(key, value)| Some((persist(key)?, persist(value)?)))
                .collect::<Option<_>>()?,
        ),
        Value::Record(record) => StaticValue::Record(
            record
                .body()
                .layout
                .fields
                .iter()
                .zip(&record.body().values)
                .map(|(name, value)| Some((name.clone(), persist(value)?)))
                .collect::<Option<_>>()?,
        ),
        Value::Enum(enumeration) => StaticValue::EnumVariant(
            enumeration.body().layout.variant.clone(),
            enumeration
                .body()
                .layout
                .fields
                .iter()
                .zip(&enumeration.body().values)
                .map(|(name, value)| Some((name.clone(), persist(value)?)))
                .collect::<Option<_>>()?,
        ),
        Value::OptionSome(value) => StaticValue::OptionSome(Box::new(persist(value)?)),
        Value::OptionNone => StaticValue::OptionNone,
        Value::ResultOk(value) => StaticValue::ResultOk(Box::new(persist(value)?)),
        Value::ResultError(value) => StaticValue::ResultError(Box::new(persist(value)?)),
        scalar => StaticValue::Scalar(to_interface(to_scalar(scalar)?)?),
    })
}

/// Restore checked data; semantic typing establishes matching nominal identities.
pub(super) fn restore(value: &StaticValue) -> Option<Value> {
    Some(match value {
        StaticValue::Scalar(value) => from_scalar(from_interface(value)?),
        StaticValue::Array(values) => Value::Array(
            values
                .iter()
                .map(restore)
                .collect::<Option<Vec<_>>>()?
                .into(),
        ),
        StaticValue::Dictionary(pairs) => Value::dict(
            pairs
                .iter()
                .map(|(key, value)| Some((restore(key)?, restore(value)?)))
                .collect::<Option<_>>()?,
        ),
        StaticValue::Record(fields) => Value::Record(SharedRecord::new(
            Arc::new(RuntimeRecordLayout {
                record: RecordTypeId::new(0),
                type_name: String::new(),
                fields: fields
                    .iter()
                    .map(|(name, _)| name.to_ascii_lowercase())
                    .collect(),
            }),
            fields
                .iter()
                .map(|(_, value)| restore(value))
                .collect::<Option<_>>()?,
        )),
        StaticValue::EnumVariant(name, fields) => Value::Enum(SharedEnum::new(
            Arc::new(RuntimeEnumLayout {
                enumeration: EnumTypeId::new(0),
                variant_id: EnumVariantId::new(0),
                type_name: String::new(),
                variant: name.to_ascii_lowercase(),
                fields: fields
                    .iter()
                    .map(|(name, _)| name.to_ascii_lowercase())
                    .collect(),
            }),
            fields
                .iter()
                .map(|(_, value)| restore(value))
                .collect::<Option<_>>()?,
        )),
        StaticValue::OptionSome(value) => Value::option_some(restore(value)?),
        StaticValue::OptionNone => Value::OptionNone,
        StaticValue::ResultOk(value) => Value::result_ok(restore(value)?),
        StaticValue::ResultError(value) => Value::result_error(restore(value)?),
    })
}
