//! Copy-on-write array value storage.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use super::Value;
use super::managed_heap::{
    clone_values, managed_value_buffer, recycle_plain_values, recycle_values,
};
use crate::resources::Reservation;

/// Copy-on-write storage for FPAS array values.
///
/// Cloning an array shares its elements until a mutable operation occurs. This preserves FPAS
/// value semantics while avoiding deep copies for ordinary reads of large arrays.
#[derive(Debug)]
struct ArrayValue {
    memory: Reservation,
    values: Vec<Value>,
    plain_data: bool,
    ownership_neutral: bool,
}

impl Clone for ArrayValue {
    fn clone(&self) -> Self {
        let (memory, values) = self.memory.copy_buffer(
            &self.values,
            crate::resources::shared_size::<Self>(0),
            clone_values,
        );
        Self {
            memory,
            values,
            plain_data: self.plain_data,
            ownership_neutral: self.ownership_neutral,
        }
    }
}

impl Drop for ArrayValue {
    fn drop(&mut self) {
        if self.memory.is_tracked() {
            super::drain::values(&mut self.values);
            return;
        }
        // Plain leaves own no nested values, so ordinary clearing cannot recurse deeply.
        if self.plain_data {
            recycle_plain_values(&mut self.values);
        } else {
            recycle_values(&mut self.values);
        }
    }
}

/// Copy-on-write storage for FPAS array values.
///
/// Final owners recycle bounded element buffers through the current thread's managed runtime
/// heap.
#[derive(Debug, Clone)]
pub struct SharedArray(Arc<ArrayValue>);

impl SharedArray {
    pub(crate) fn allocation_identity(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }

    /// Borrow the elements without exposing unchecked vector growth.
    pub fn as_slice(&self) -> &[Value] {
        &self.0.values
    }

    /// Insert an element, reserving storage before moving the suffix.
    pub fn insert(&mut self, index: usize, value: Value) {
        let body = Arc::make_mut(&mut self.0);
        body.memory.reserve_vec(&mut body.values, 1);
        body.plain_data &= is_plain_data_leaf(&value);
        body.ownership_neutral &= value.is_ownership_neutral();
        body.values.insert(index, value);
    }
    /// Whether every element is a primitive value without ownership or capabilities.
    ///
    /// A true result guarantees direct integer, real, boolean, string, unit or absent-option
    /// elements only. The check is constant-time; false may conservatively remain after mutation.
    #[must_use]
    pub fn is_plain_data(&self) -> bool {
        self.0.plain_data
    }

    pub(super) fn is_ownership_neutral(&self) -> bool {
        self.0.ownership_neutral
    }

    /// Append an element while preserving known primitive-only contents.
    #[inline]
    pub fn push(&mut self, value: Value) {
        let body = Arc::make_mut(&mut self.0);
        body.memory.reserve_vec(&mut body.values, 1);
        body.plain_data &= is_plain_data_leaf(&value);
        body.ownership_neutral &= value.is_ownership_neutral();
        body.values.push(value);
    }

    /// Remove the last element without invalidating known primitive-only contents.
    pub fn pop(&mut self) -> Option<Value> {
        Arc::make_mut(&mut self.0).values.pop()
    }

    /// Append values while reserving their backing storage before growth.
    pub fn extend(&mut self, values: impl IntoIterator<Item = Value>) {
        for value in values {
            self.push(value);
        }
    }

    /// Keep the first `length` elements, retaining the allocated capacity.
    pub fn truncate(&mut self, length: usize) {
        Arc::make_mut(&mut self.0).values.truncate(length);
    }

    /// Remove all elements, retaining the allocated capacity.
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Remove an element and shift the remaining suffix.
    pub fn remove(&mut self, index: usize) -> Value {
        Arc::make_mut(&mut self.0).values.remove(index)
    }

    /// Retain elements selected by a predicate.
    pub fn retain(&mut self, predicate: impl FnMut(&Value) -> bool) {
        Arc::make_mut(&mut self.0).values.retain(predicate);
    }

    /// Replace one element and return its previous value.
    ///
    /// # Panics
    /// Panics if `index` is outside the array.
    pub fn replace_element(&mut self, index: usize, value: Value) -> Value {
        let body = Arc::make_mut(&mut self.0);
        body.plain_data &= is_plain_data_leaf(&value);
        body.ownership_neutral &= value.is_ownership_neutral();
        std::mem::replace(&mut body.values[index], value)
    }

    /// Edit one element, retaining cached graph properties when the result permits them.
    ///
    /// A panicking callback leaves the properties invalidated.
    ///
    /// # Panics
    /// Panics if `index` is outside the array, or if the callback panics.
    pub fn update_element<R>(&mut self, index: usize, update: impl FnOnce(&mut Value) -> R) -> R {
        let body = Arc::make_mut(&mut self.0);
        let plain_data = std::mem::replace(&mut body.plain_data, false);
        let ownership_neutral = std::mem::replace(&mut body.ownership_neutral, false);
        let result = update(&mut body.values[index]);
        body.plain_data = plain_data && is_plain_data_leaf(&body.values[index]);
        body.ownership_neutral = ownership_neutral && body.values[index].is_ownership_neutral();
        result
    }

    pub(super) fn drain_unique(self, pending: &mut Vec<Value>) {
        if let Some(mut body) = Arc::into_inner(self.0)
            && !body.plain_data
        {
            pending.append(&mut body.values);
        }
        // A plain body is dropped here and cleared flat.
    }
}

impl From<Vec<Value>> for SharedArray {
    fn from(values: Vec<Value>) -> Self {
        // Plain leaves are ownership-neutral, so most arrays need only the first scan.
        let plain_data = values.iter().all(is_plain_data_leaf);
        let ownership_neutral = plain_data || values.iter().all(Value::is_ownership_neutral);
        Self(Arc::new(ArrayValue {
            memory: Reservation::shared::<ArrayValue>(
                values.capacity() * std::mem::size_of::<Value>(),
            ),
            values,
            plain_data,
            ownership_neutral,
        }))
    }
}

impl FromIterator<Value> for SharedArray {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        if crate::resources::heap_active() {
            let mut array = Self::from(Vec::new());
            array.extend(iter);
            return array;
        }
        let iter = iter.into_iter();
        let mut values = managed_value_buffer(iter.size_hint().0);
        values.extend(iter);
        Self::from(values)
    }
}

impl From<SharedArray> for Vec<Value> {
    fn from(values: SharedArray) -> Self {
        match Arc::try_unwrap(values.0) {
            Ok(mut body) => std::mem::take(&mut body.values),
            Err(body) => clone_values(&body.values),
        }
    }
}

impl Deref for SharedArray {
    type Target = [Value];

    fn deref(&self) -> &Self::Target {
        &self.0.values
    }
}

impl DerefMut for SharedArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let body = Arc::make_mut(&mut self.0);
        body.plain_data = false;
        body.ownership_neutral = false;
        &mut body.values
    }
}

fn is_plain_data_leaf(value: &Value) -> bool {
    matches!(
        value,
        Value::Integer(_)
            | Value::Real(_)
            | Value::Boolean(_)
            | Value::Str(_)
            | Value::Unit
            | Value::OptionNone
    )
}

impl<'a> IntoIterator for &'a SharedArray {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for SharedArray {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        Vec::from(self).into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_copies_only_when_mutated() {
        let original = SharedArray::from(vec![Value::Integer(1)]);
        let mut updated = original.clone();
        assert!(Arc::ptr_eq(&original.0, &updated.0));

        updated[0] = Value::Integer(2);

        assert!(!Arc::ptr_eq(&original.0, &updated.0));
        assert_eq!(original[0], Value::Integer(1));
        assert_eq!(updated[0], Value::Integer(2));
        assert!(original.is_plain_data());
        assert!(!updated.is_plain_data());
    }

    #[test]
    fn primitive_contents_remain_known_through_specialized_mutations() {
        let mut values = SharedArray::from(vec![
            Value::Integer(1),
            Value::Real(2.0),
            Value::Boolean(true),
            Value::Str("three".into()),
            Value::Unit,
            Value::OptionNone,
        ]);
        assert!(values.is_plain_data());
        let original = values.clone();
        values.push(Value::Integer(4));
        assert!(!Arc::ptr_eq(&original.0, &values.0));
        assert!(values.is_plain_data());
        assert!(original.is_plain_data());
        assert_eq!(values.pop(), Some(Value::Integer(4)));
        assert_eq!(values.replace_element(0, Value::Unit), Value::Integer(1));
        assert!(values.is_plain_data());
        assert_eq!(original[0], Value::Integer(1));
        assert!(SharedArray::from(Vec::new()).is_plain_data());
    }

    #[test]
    fn capability_and_wrapped_values_are_never_marked_as_plain_data() {
        let domain = super::super::MemoryDomain::new();
        let candidates = [
            Value::Task(1),
            Value::OpaqueHandle(2),
            Value::Cell(domain.new_cell(Value::Unit).unwrap()),
            Value::Reference(Arc::new(super::super::VariableReference {
                root: super::super::ReferenceRoot::Global {
                    slot: 0,
                    owner: domain.id().clone(),
                },
                path: Vec::new(),
            })),
            Value::option_some(Value::Unit),
            Value::result_ok(Value::Unit),
            Value::result_error(Value::Unit),
            Value::Array(Vec::new().into()),
            Value::dict(Vec::new()),
        ];
        for value in candidates {
            assert!(!SharedArray::from(vec![value.clone()]).is_plain_data());
            let original = SharedArray::from(vec![Value::Unit]);
            let mut appended = original.clone();
            appended.push(value.clone());
            assert!(!appended.is_plain_data());
            assert!(original.is_plain_data());
            let _ = appended.pop();
            assert!(!appended.is_plain_data());

            let mut replaced = original.clone();
            replaced.replace_element(0, value);
            assert!(!replaced.is_plain_data());
            assert!(original.is_plain_data());
            replaced.replace_element(0, Value::Unit);
            assert!(!replaced.is_plain_data());
        }
    }

    #[test]
    fn raw_mutable_access_invalidates_metadata_before_any_change() {
        let original = SharedArray::from(vec![Value::Integer(1)]);
        let mut edited = original.clone();
        let raw: &mut [Value] = edited.deref_mut();
        raw[0] = Value::OpaqueHandle(2);
        assert!(!edited.is_plain_data());
        assert!(original.is_plain_data());
        let clone = edited.clone();
        assert!(!clone.is_plain_data());

        let mut unchanged = original.clone();
        let _ = unchanged.deref_mut();
        assert!(!unchanged.is_plain_data());
        assert!(original.is_plain_data());
    }
}
