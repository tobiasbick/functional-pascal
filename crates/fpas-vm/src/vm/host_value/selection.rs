//! Inspection of immutable aggregate children without exposing local graphs.

use std::sync::Arc;

use fpas_bytecode::{EnumTypeId, EnumVariantId, Value};

use super::{HostValue, HostValueError};

/// One direct immutable child of a returned value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostChild<'a> {
    /// Array item or positional record/enum field.
    Index(usize),
    /// Named record or enum field, using case-insensitive FPAS field names.
    Field(&'a str),
    /// Dictionary key in insertion order.
    DictKey(usize),
    /// Dictionary value in insertion order.
    DictValue(usize),
    /// Payload of `Some`, `Ok`, or `Error`.
    Payload,
    /// A callable's captured value, retained as a sealed child.
    Capture(usize),
    /// A callable's bound receiver.
    Receiver,
}

/// Detached enum layout information without associated runtime values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEnumInfo {
    /// Numeric enum identity in the exporting executable.
    pub enumeration: EnumTypeId,
    /// Numeric variant identity in the exporting executable.
    pub variant_id: EnumVariantId,
    /// Canonical enum type name.
    pub type_name: String,
    /// Canonical variant name.
    pub variant: String,
    /// Associated field names in declaration order.
    pub fields: Vec<String>,
}

impl HostValue {
    /// Select an aggregate child while preserving its session provenance.
    ///
    /// Returns `None` for an unavailable child. Callable captures and bound receivers
    /// return registered sealed children. Read cell payloads with `HostValue::read_cell`;
    /// references are not traversed.
    ///
    /// # Errors
    /// Returns an error when a retained value cannot be accessed.
    pub fn child(&self, selector: HostChild<'_>) -> Result<Option<Self>, HostValueError> {
        let _access = self.0.origin.domain.access();
        let child = match (&self.0.value, selector) {
            (Value::Function(function), HostChild::Capture(index)) => function.captures.get(index),
            (Value::Function(function), HostChild::Receiver) => function.bound_receiver.as_ref(),
            (Value::Array(values), HostChild::Index(index)) => values.get(index),
            (Value::Record(record), HostChild::Index(index)) => record.body().values.get(index),
            (Value::Enum(enumeration), HostChild::Index(index)) => {
                enumeration.body().values.get(index)
            }
            (Value::Record(record), HostChild::Field(name)) => record
                .body()
                .layout
                .fields
                .iter()
                .position(|field| field.eq_ignore_ascii_case(name))
                .and_then(|index| record.body().values.get(index)),
            (Value::Enum(enumeration), HostChild::Field(name)) => enumeration
                .body()
                .layout
                .fields
                .iter()
                .position(|field| field.eq_ignore_ascii_case(name))
                .and_then(|index| enumeration.body().values.get(index)),
            (Value::Dict(pairs), HostChild::DictKey(index)) => pairs.get(index).map(|pair| &pair.0),
            (Value::Dict(pairs), HostChild::DictValue(index)) => {
                pairs.get(index).map(|pair| &pair.1)
            }
            (
                Value::OptionSome(payload) | Value::ResultOk(payload) | Value::ResultError(payload),
                HostChild::Payload,
            ) => Some(payload.as_ref()),
            _ => None,
        };
        Ok(child.map(|value| Self::new(value.clone(), Arc::clone(&self.0.origin))))
    }

    /// Inspect an enum's detached layout without exposing its associated values.
    #[must_use]
    pub fn enum_info(&self) -> Option<HostEnumInfo> {
        let _access = self.0.origin.domain.access();
        let Value::Enum(value) = &self.0.value else {
            return None;
        };
        let layout = &value.body().layout;
        Some(HostEnumInfo {
            enumeration: layout.enumeration,
            variant_id: layout.variant_id,
            type_name: layout.type_name.clone(),
            variant: layout.variant.clone(),
            fields: layout.fields.clone(),
        })
    }
}
