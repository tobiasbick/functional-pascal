//! Copy-on-write storage for compound FPAS values.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use super::Value;
use super::managed_heap::{
    clone_pairs, clone_values, recycle_error_values, recycle_pairs, recycle_values,
};
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
#[derive(Debug)]
pub struct RecordValue {
    /// Shared layout metadata.
    pub layout: Arc<RuntimeRecordLayout>,
    /// Values in layout order.
    pub values: Vec<Value>,
}

impl Clone for RecordValue {
    fn clone(&self) -> Self {
        Self {
            layout: Arc::clone(&self.layout),
            values: clone_values(&self.values),
        }
    }
}

impl Drop for RecordValue {
    fn drop(&mut self) {
        recycle_values(&mut self.values);
    }
}

/// Compact copy-on-write record value.
#[derive(Debug, Clone)]
pub struct SharedRecord(Arc<RecordValue>);

impl SharedRecord {
    pub(super) fn drain_unique(self, pending: &mut Vec<Value>) {
        if let Some(mut body) = Arc::into_inner(self.0) {
            pending.append(&mut body.values);
        }
    }
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
    /// Automatic error rendering and exact Cause identity, absent for ordinary enums.
    pub error: Option<RuntimeErrorVariant>,
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

/// Shared error information resolved from admitted executable metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeErrorVariant {
    /// Optional literal message, without formatting placeholders.
    pub message: Option<String>,
    /// Exact original cause field and family.
    pub cause: Option<crate::ErrorCause>,
}

impl RuntimeErrorVariant {
    /// Resolve checked literal references once while building shared runtime layouts.
    ///
    /// # Errors
    /// Rejects an unavailable label, indicating an unadmitted or inconsistent image.
    pub fn from_metadata(
        error: &crate::ErrorVariant,
        strings: &crate::StringTable,
    ) -> Result<Self, EnumConstructionError> {
        Ok(Self {
            message: error
                .message
                .map(|id| {
                    strings
                        .get(id)
                        .map(str::to_owned)
                        .ok_or(EnumConstructionError("error message string is unavailable"))
                })
                .transpose()?,
            cause: error.cause,
        })
    }
}

/// A value does not match its admitted enum construction contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumConstructionError(pub &'static str);

impl std::fmt::Display for EnumConstructionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for EnumConstructionError {}

/// Stored enum body.
#[derive(Debug)]
pub struct EnumValue {
    /// Shared variant layout metadata.
    pub layout: Arc<RuntimeEnumLayout>,
    /// Associated values in declaration order.
    pub values: Vec<Value>,
}

impl Clone for EnumValue {
    fn clone(&self) -> Self {
        Self {
            layout: Arc::clone(&self.layout),
            values: clone_values(&self.values),
        }
    }
}

impl Drop for EnumValue {
    fn drop(&mut self) {
        if self.layout.error.is_some() {
            recycle_error_values(&mut self.values);
        } else {
            recycle_values(&mut self.values);
        }
    }
}

/// Compact shared enum value.
#[derive(Debug, Clone)]
pub struct SharedEnum(Arc<EnumValue>);

impl SharedEnum {
    pub(super) fn drain_unique(self, pending: &mut Vec<Value>) {
        if let Some(mut body) = Arc::into_inner(self.0) {
            pending.append(&mut body.values);
        }
    }
    /// Construct a value after checking arity and its original typed Cause.
    ///
    /// # Errors
    /// Rejects wrong field counts or a Cause from another concrete error instance.
    pub fn try_new(
        layout: Arc<RuntimeEnumLayout>,
        values: Vec<Value>,
    ) -> Result<Self, EnumConstructionError> {
        if layout.fields.len() != values.len() {
            return Err(EnumConstructionError(
                "enum constructor field count does not match its layout",
            ));
        }
        if let Some(cause) = layout.error.as_ref().and_then(|error| error.cause.as_ref())
            && !matches!(values.get(usize::from(cause.field)), Some(Value::Enum(value))
                if value.body().layout.enumeration == cause.family && value.body().layout.error.is_some())
        {
            return Err(EnumConstructionError(
                "error Cause does not match its concrete error family",
            ));
        }
        Ok(Self::new(layout, values))
    }

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

    /// Mutably borrow values, detaching only when the body is shared.
    pub fn values_mut(&mut self) -> &mut Vec<Value> {
        &mut Arc::make_mut(&mut self.0).values
    }
}

#[derive(Debug)]
struct DictValue {
    pairs: Vec<(Value, Value)>,
}

impl Clone for DictValue {
    fn clone(&self) -> Self {
        Self {
            pairs: clone_pairs(&self.pairs),
        }
    }
}

impl Drop for DictValue {
    fn drop(&mut self) {
        recycle_pairs(&mut self.pairs);
    }
}

/// Copy-on-write storage for ordered dictionary pairs.
///
/// Final owners recycle bounded pair buffers through the current thread's managed runtime heap.
#[derive(Debug, Clone)]
pub struct SharedDict(Arc<DictValue>);

impl From<Vec<(Value, Value)>> for SharedDict {
    fn from(pairs: Vec<(Value, Value)>) -> Self {
        Self(Arc::new(DictValue { pairs }))
    }
}

impl SharedDict {
    pub(super) fn drain_unique(self, pending: &mut Vec<Value>) {
        if let Some(mut body) = Arc::into_inner(self.0) {
            for (key, value) in body.pairs.drain(..) {
                pending.push(key);
                pending.push(value);
            }
        }
    }
}

impl From<SharedDict> for Vec<(Value, Value)> {
    fn from(pairs: SharedDict) -> Self {
        match Arc::try_unwrap(pairs.0) {
            Ok(mut body) => std::mem::take(&mut body.pairs),
            Err(body) => clone_pairs(&body.pairs),
        }
    }
}

impl Deref for SharedDict {
    type Target = Vec<(Value, Value)>;

    fn deref(&self) -> &Self::Target {
        &self.0.pairs
    }
}

impl DerefMut for SharedDict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut Arc::make_mut(&mut self.0).pairs
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

    #[test]
    fn cloned_enum_detaches_on_mutation() {
        let layout = Arc::new(RuntimeEnumLayout {
            error: None,
            enumeration: EnumTypeId::new(0),
            variant_id: EnumVariantId::new(0),
            type_name: "Choice".to_string(),
            variant: "Count".to_string(),
            fields: vec!["Value".to_string()],
        });
        let original = SharedEnum::new(layout, vec![Value::Integer(1)]);
        let mut updated = original.clone();
        updated.values_mut()[0] = Value::Integer(2);
        assert_eq!(original.body().values[0], Value::Integer(1));
        assert_eq!(updated.body().values[0], Value::Integer(2));
    }
}
