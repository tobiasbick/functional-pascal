//! Copy-on-write storage for compound FPAS values.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use super::Value;
use super::managed_heap::{
    clone_pairs, clone_values, recycle_error_values, recycle_pairs, recycle_values,
};
use crate::resources::Reservation;
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
    memory: Reservation,
    /// Shared layout metadata.
    pub layout: Arc<RuntimeRecordLayout>,
    /// Values in layout order.
    pub values: Vec<Value>,
    ownership_neutral: bool,
}

impl Clone for RecordValue {
    fn clone(&self) -> Self {
        let (memory, values) = self.memory.copy_buffer(
            &self.values,
            crate::resources::shared_size::<Self>(0),
            clone_values,
        );
        Self {
            memory,
            layout: Arc::clone(&self.layout),
            values,
            ownership_neutral: self.ownership_neutral,
        }
    }
}

impl Drop for RecordValue {
    fn drop(&mut self) {
        if self.memory.is_tracked() {
            super::drain::values(&mut self.values);
            return;
        }
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
        let ownership_neutral = values.iter().all(Value::is_ownership_neutral);
        Self(Arc::new(RecordValue {
            memory: Reservation::shared::<RecordValue>(
                values.capacity() * std::mem::size_of::<Value>(),
            ),
            layout,
            values,
            ownership_neutral,
        }))
    }

    /// Borrow the immutable body.
    #[must_use]
    pub fn body(&self) -> &RecordValue {
        &self.0
    }

    /// Mutably borrow values, detaching only when the body is shared.
    pub fn values_mut(&mut self) -> &mut [Value] {
        let body = Arc::make_mut(&mut self.0);
        body.ownership_neutral = false;
        &mut body.values
    }

    pub(super) fn is_ownership_neutral(&self) -> bool {
        self.0.ownership_neutral
    }

    /// Replace a field while preserving known ownership properties.
    ///
    /// # Panics
    /// Panics if `index` is outside the stored fields.
    #[inline]
    pub fn replace_field(&mut self, index: usize, value: Value) -> Value {
        let body = Arc::make_mut(&mut self.0);
        body.ownership_neutral &= value.is_ownership_neutral();
        std::mem::replace(&mut body.values[index], value)
    }

    /// Edit one field, invalidating cached ownership properties if the callback panics.
    ///
    /// # Panics
    /// Panics if `index` is outside the stored fields, or if the callback panics.
    pub fn update_field<R>(&mut self, index: usize, update: impl FnOnce(&mut Value) -> R) -> R {
        let body = Arc::make_mut(&mut self.0);
        let ownership_neutral = std::mem::replace(&mut body.ownership_neutral, false);
        let result = update(&mut body.values[index]);
        body.ownership_neutral = ownership_neutral && body.values[index].is_ownership_neutral();
        result
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
    /// Native display policy; associated values remain available for explicit inspection.
    pub hide_payload: bool,
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
            hide_payload: false,
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
    memory: Reservation,
    /// Shared variant layout metadata.
    pub layout: Arc<RuntimeEnumLayout>,
    /// Associated values in declaration order.
    pub values: Vec<Value>,
    ownership_neutral: bool,
}

impl Clone for EnumValue {
    fn clone(&self) -> Self {
        let (memory, values) = self.memory.copy_buffer(
            &self.values,
            crate::resources::shared_size::<Self>(0),
            clone_values,
        );
        Self {
            memory,
            layout: Arc::clone(&self.layout),
            values,
            ownership_neutral: self.ownership_neutral,
        }
    }
}

impl Drop for EnumValue {
    fn drop(&mut self) {
        if self.memory.is_tracked() {
            super::drain::values(&mut self.values);
            return;
        }
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
        let ownership_neutral = values.iter().all(Value::is_ownership_neutral);
        Self(Arc::new(EnumValue {
            memory: Reservation::shared::<EnumValue>(
                values.capacity() * std::mem::size_of::<Value>(),
            ),
            layout,
            values,
            ownership_neutral,
        }))
    }

    /// Borrow the immutable body.
    #[must_use]
    pub fn body(&self) -> &EnumValue {
        &self.0
    }

    /// Mutably borrow values, detaching only when the body is shared.
    pub fn values_mut(&mut self) -> &mut [Value] {
        let body = Arc::make_mut(&mut self.0);
        body.ownership_neutral = false;
        &mut body.values
    }

    pub(super) fn is_ownership_neutral(&self) -> bool {
        self.0.ownership_neutral
    }

    /// Replace a field while preserving known ownership properties.
    ///
    /// # Panics
    /// Panics if `index` is outside the stored fields.
    pub fn replace_field(&mut self, index: usize, value: Value) -> Value {
        let body = Arc::make_mut(&mut self.0);
        body.ownership_neutral &= value.is_ownership_neutral();
        std::mem::replace(&mut body.values[index], value)
    }

    /// Edit one field, invalidating cached ownership properties if the callback panics.
    ///
    /// # Panics
    /// Panics if `index` is outside the stored fields, or if the callback panics.
    pub fn update_field<R>(&mut self, index: usize, update: impl FnOnce(&mut Value) -> R) -> R {
        let body = Arc::make_mut(&mut self.0);
        let ownership_neutral = std::mem::replace(&mut body.ownership_neutral, false);
        let result = update(&mut body.values[index]);
        body.ownership_neutral = ownership_neutral && body.values[index].is_ownership_neutral();
        result
    }
}

#[derive(Debug)]
struct DictValue {
    memory: Reservation,
    pairs: Vec<(Value, Value)>,
    ownership_neutral: bool,
}

impl Clone for DictValue {
    fn clone(&self) -> Self {
        let (memory, pairs) = self.memory.copy_buffer(
            &self.pairs,
            crate::resources::shared_size::<Self>(0),
            clone_pairs,
        );
        Self {
            memory,
            pairs,
            ownership_neutral: self.ownership_neutral,
        }
    }
}

impl Drop for DictValue {
    fn drop(&mut self) {
        if self.memory.is_tracked() {
            super::drain::pairs(&mut self.pairs);
            return;
        }
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
        let ownership_neutral = pairs
            .iter()
            .all(|(key, value)| key.is_ownership_neutral() && value.is_ownership_neutral());
        Self(Arc::new(DictValue {
            memory: Reservation::shared::<DictValue>(
                pairs.capacity() * std::mem::size_of::<(Value, Value)>(),
            ),
            pairs,
            ownership_neutral,
        }))
    }
}

impl SharedDict {
    pub(crate) fn allocation_identity(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }

    pub(super) fn is_ownership_neutral(&self) -> bool {
        self.0.ownership_neutral
    }

    /// Append a pair while preserving known ownership properties of both key and value.
    pub fn push(&mut self, pair: (Value, Value)) {
        let body = Arc::make_mut(&mut self.0);
        body.memory.reserve_vec(&mut body.pairs, 1);
        body.ownership_neutral &= pair.0.is_ownership_neutral() && pair.1.is_ownership_neutral();
        body.pairs.push(pair);
    }

    /// Remove a pair and shift the remaining suffix.
    pub fn remove(&mut self, index: usize) -> (Value, Value) {
        Arc::make_mut(&mut self.0).pairs.remove(index)
    }

    /// Remove all pairs while retaining their backing capacity.
    pub fn clear(&mut self) {
        Arc::make_mut(&mut self.0).pairs.clear();
    }

    /// Replace the value at one pair position, retaining its key.
    ///
    /// # Panics
    /// Panics if `index` is outside the stored pairs.
    pub fn replace_value(&mut self, index: usize, value: Value) -> Value {
        let body = Arc::make_mut(&mut self.0);
        body.ownership_neutral &= value.is_ownership_neutral();
        std::mem::replace(&mut body.pairs[index].1, value)
    }

    /// Edit one value, invalidating cached ownership properties if the callback panics.
    ///
    /// # Panics
    /// Panics if `index` is outside the stored pairs, or if the callback panics.
    pub fn update_value<R>(&mut self, index: usize, update: impl FnOnce(&mut Value) -> R) -> R {
        let body = Arc::make_mut(&mut self.0);
        let ownership_neutral = std::mem::replace(&mut body.ownership_neutral, false);
        let result = update(&mut body.pairs[index].1);
        body.ownership_neutral = ownership_neutral && body.pairs[index].1.is_ownership_neutral();
        result
    }

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
    type Target = [(Value, Value)];

    fn deref(&self) -> &Self::Target {
        &self.0.pairs
    }
}

impl DerefMut for SharedDict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let body = Arc::make_mut(&mut self.0);
        body.ownership_neutral = false;
        &mut body.pairs
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
mod tests;
