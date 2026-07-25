//! Copy-on-write array value storage.

use super::Value;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Copy-on-write storage for FPAS array values.
///
/// Cloning an array shares its elements until a mutable operation occurs. This preserves FPAS
/// value semantics while avoiding deep copies for ordinary reads of large arrays.
#[derive(Debug, Clone)]
pub struct SharedArray(Arc<Vec<Value>>);

impl From<Vec<Value>> for SharedArray {
    fn from(values: Vec<Value>) -> Self {
        Self(Arc::new(values))
    }
}

impl FromIterator<Value> for SharedArray {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl From<SharedArray> for Vec<Value> {
    fn from(values: SharedArray) -> Self {
        Arc::unwrap_or_clone(values.0)
    }
}

impl Deref for SharedArray {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SharedArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
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
        Arc::unwrap_or_clone(self.0).into_iter()
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
