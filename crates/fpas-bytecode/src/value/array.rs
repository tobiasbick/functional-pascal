//! Copy-on-write array value storage.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use super::Value;
use super::managed_heap::{clone_values, managed_value_buffer, recycle_values};

/// Copy-on-write storage for FPAS array values.
///
/// Cloning an array shares its elements until a mutable operation occurs. This preserves FPAS
/// value semantics while avoiding deep copies for ordinary reads of large arrays.
#[derive(Debug)]
struct ArrayValue {
    values: Vec<Value>,
}

impl Clone for ArrayValue {
    fn clone(&self) -> Self {
        Self {
            values: clone_values(&self.values),
        }
    }
}

impl Drop for ArrayValue {
    fn drop(&mut self) {
        recycle_values(&mut self.values);
    }
}

/// Copy-on-write storage for FPAS array values.
///
/// Final owners recycle bounded element buffers through the current thread's managed runtime
/// heap.
#[derive(Debug, Clone)]
pub struct SharedArray(Arc<ArrayValue>);

impl From<Vec<Value>> for SharedArray {
    fn from(values: Vec<Value>) -> Self {
        Self(Arc::new(ArrayValue { values }))
    }
}

impl FromIterator<Value> for SharedArray {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
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
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0.values
    }
}

impl DerefMut for SharedArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut Arc::make_mut(&mut self.0).values
    }
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
    }
}
