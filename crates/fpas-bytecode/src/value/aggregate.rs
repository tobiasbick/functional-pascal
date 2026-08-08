//! Copy-on-write storage for compound FPAS values.

use super::Value;
use crate::{EnumTypeId, EnumVariantId, RecordTypeId};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Shared immutable names for one positional record layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionalRecordLayout {
    /// Numeric executable-local record identity.
    pub record: RecordTypeId,
    /// Canonical runtime type name.
    pub type_name: String,
    /// Canonical field names in numeric slot order.
    pub fields: Vec<String>,
}

/// Stored positional record body with copy-on-write values.
#[derive(Debug, Clone)]
pub struct PositionalRecordValue {
    /// Shared layout metadata.
    pub layout: Arc<PositionalRecordLayout>,
    /// Values in layout order.
    pub values: Vec<Value>,
}

/// Compact shared positional record value used by register bytecode.
#[derive(Debug, Clone)]
pub struct SharedPositionalRecord(Arc<PositionalRecordValue>);

impl SharedPositionalRecord {
    /// Construct a positional record from shared layout metadata.
    pub fn new(layout: Arc<PositionalRecordLayout>, values: Vec<Value>) -> Self {
        Self(Arc::new(PositionalRecordValue { layout, values }))
    }

    /// Borrow the immutable body.
    #[must_use]
    pub fn body(&self) -> &PositionalRecordValue {
        &self.0
    }

    /// Mutably borrow values, detaching only when the body is shared.
    pub fn values_mut(&mut self) -> &mut Vec<Value> {
        &mut Arc::make_mut(&mut self.0).values
    }
}

/// Shared immutable names and numeric identity for one enum variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionalEnumLayout {
    /// Numeric executable-local enum identity.
    pub enumeration: EnumTypeId,
    /// Numeric executable-wide variant identity.
    pub variant_id: EnumVariantId,
    /// Canonical enum type name.
    pub type_name: String,
    /// Canonical variant name.
    pub variant: String,
    /// Associated field names in slot order.
    pub fields: Vec<String>,
}

/// Stored positional enum body.
#[derive(Debug, Clone)]
pub struct PositionalEnumValue {
    /// Shared variant layout metadata.
    pub layout: Arc<PositionalEnumLayout>,
    /// Associated values in declaration order.
    pub values: Vec<Value>,
}

/// Compact shared positional enum value used by register bytecode.
#[derive(Debug, Clone)]
pub struct SharedPositionalEnum(Arc<PositionalEnumValue>);

impl SharedPositionalEnum {
    /// Construct a positional enum from shared layout metadata.
    pub fn new(layout: Arc<PositionalEnumLayout>, values: Vec<Value>) -> Self {
        Self(Arc::new(PositionalEnumValue { layout, values }))
    }

    /// Borrow the immutable body.
    #[must_use]
    pub fn body(&self) -> &PositionalEnumValue {
        &self.0
    }
}

/// Stored enum data shared by cloned enum values.
#[derive(Debug, Clone)]
pub struct EnumValue {
    /// Canonical runtime enum type name.
    pub type_name: String,
    /// Active variant name.
    pub variant: String,
    /// Associated variant fields in declaration order.
    pub fields: Vec<Value>,
}

/// Shared immutable storage for an enum value.
#[derive(Debug, Clone)]
pub struct SharedEnum(Arc<EnumValue>);

impl SharedEnum {
    /// Create a shared enum value.
    pub fn new(type_name: String, variant: String, fields: Vec<Value>) -> Self {
        Self(Arc::new(EnumValue {
            type_name,
            variant,
            fields,
        }))
    }

    /// Consume the wrapper, cloning the body only when other values still share it.
    pub fn into_parts(self) -> (String, String, Vec<Value>) {
        let value = Arc::unwrap_or_clone(self.0);
        (value.type_name, value.variant, value.fields)
    }
}

impl Deref for SharedEnum {
    type Target = EnumValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Copy-on-write storage for ordered dictionary pairs.
#[derive(Debug, Clone)]
pub struct SharedDict(Arc<Vec<(Value, Value)>>);

impl From<Vec<(Value, Value)>> for SharedDict {
    fn from(pairs: Vec<(Value, Value)>) -> Self {
        Self(Arc::new(pairs))
    }
}

impl From<SharedDict> for Vec<(Value, Value)> {
    fn from(pairs: SharedDict) -> Self {
        Arc::unwrap_or_clone(pairs.0)
    }
}

impl Deref for SharedDict {
    type Target = Vec<(Value, Value)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SharedDict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}

impl<'a> IntoIterator for &'a SharedDict {
    type Item = &'a (Value, Value);
    type IntoIter = std::slice::Iter<'a, (Value, Value)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for SharedDict {
    type Item = (Value, Value);
    type IntoIter = std::vec::IntoIter<(Value, Value)>;

    fn into_iter(self) -> Self::IntoIter {
        Vec::from(self).into_iter()
    }
}

/// Stored record data shared by cloned record values.
#[derive(Debug, Clone)]
pub struct RecordValue {
    /// Canonical runtime record type name.
    pub type_name: String,
    /// Record fields in declaration order.
    pub fields: Vec<(String, Value)>,
}

/// Copy-on-write storage for a record value.
#[derive(Debug, Clone)]
pub struct SharedRecord(Arc<RecordValue>);

impl SharedRecord {
    /// Create a shared record value.
    pub fn new(type_name: String, fields: Vec<(String, Value)>) -> Self {
        Self(Arc::new(RecordValue { type_name, fields }))
    }

    /// Consume the wrapper, cloning the body only when other values still share it.
    pub fn into_parts(self) -> (String, Vec<(String, Value)>) {
        let value = Arc::unwrap_or_clone(self.0);
        (value.type_name, value.fields)
    }
}

impl Deref for SharedRecord {
    type Target = RecordValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SharedRecord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloned_dict_detaches_on_mutation() {
        let original = SharedDict::from(vec![(Value::Integer(1), Value::Integer(2))]);
        let mut updated = original.clone();

        updated[0].1 = Value::Integer(3);

        assert_eq!(original[0].1, Value::Integer(2));
        assert_eq!(updated[0].1, Value::Integer(3));
    }

    #[test]
    fn cloned_record_detaches_on_mutation() {
        let original = SharedRecord::new(
            "Point".to_string(),
            vec![("x".to_string(), Value::Integer(1))],
        );
        let mut updated = original.clone();

        updated.fields[0].1 = Value::Integer(2);

        assert_eq!(original.fields[0].1, Value::Integer(1));
        assert_eq!(updated.fields[0].1, Value::Integer(2));
    }
}
