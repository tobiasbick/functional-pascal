//! Copy-on-write storage for compound FPAS values.

use super::Value;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

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
