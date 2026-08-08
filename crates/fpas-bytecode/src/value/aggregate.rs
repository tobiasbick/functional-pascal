//! Copy-on-write storage for compound FPAS values.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use super::Value;
use crate::{EnumTypeId, EnumVariantId, RecordTypeId};

/// Shared immutable metadata for one runtime record layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRecordLayout {
    /// Numeric executable-local record identity.
    pub record: RecordTypeId,
    /// Canonical runtime type name.
    pub type_name: String,
    /// Canonical field names in numeric slot order.
    pub fields: Vec<String>,
}

/// Stored record body with copy-on-write values.
#[derive(Debug, Clone)]
pub struct RecordValue {
    /// Shared layout metadata.
    pub layout: Arc<RuntimeRecordLayout>,
    /// Values in layout order.
    pub values: Vec<Value>,
}

/// Compact copy-on-write record value.
#[derive(Debug, Clone)]
pub struct SharedRecord(Arc<RecordValue>);

impl SharedRecord {
    /// Construct a record from shared layout metadata.
    #[must_use]
    pub fn new(layout: Arc<RuntimeRecordLayout>, values: Vec<Value>) -> Self {
        Self(Arc::new(RecordValue { layout, values }))
    }

    /// Borrow the immutable body.
    #[must_use]
    pub fn body(&self) -> &RecordValue {
        &self.0
    }

    /// Mutably borrow values, detaching only when the body is shared.
    pub fn values_mut(&mut self) -> &mut Vec<Value> {
        &mut Arc::make_mut(&mut self.0).values
    }
}

/// Shared immutable metadata for one runtime enum variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEnumLayout {
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

/// Stored enum body.
#[derive(Debug, Clone)]
pub struct EnumValue {
    /// Shared variant layout metadata.
    pub layout: Arc<RuntimeEnumLayout>,
    /// Associated values in declaration order.
    pub values: Vec<Value>,
}

/// Compact shared enum value.
#[derive(Debug, Clone)]
pub struct SharedEnum(Arc<EnumValue>);

impl SharedEnum {
    /// Construct an enum from shared variant metadata.
    #[must_use]
    pub fn new(layout: Arc<RuntimeEnumLayout>, values: Vec<Value>) -> Self {
        Self(Arc::new(EnumValue { layout, values }))
    }

    /// Borrow the immutable body.
    #[must_use]
    pub fn body(&self) -> &EnumValue {
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
        let layout = Arc::new(RuntimeRecordLayout {
            record: RecordTypeId::new(0),
            type_name: "Point".to_string(),
            fields: vec!["x".to_string()],
        });
        let original = SharedRecord::new(layout, vec![Value::Integer(1)]);
        let mut updated = original.clone();
        updated.values_mut()[0] = Value::Integer(2);
        assert_eq!(original.body().values[0], Value::Integer(1));
        assert_eq!(updated.body().values[0], Value::Integer(2));
    }
}
