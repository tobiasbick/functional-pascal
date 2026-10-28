//! One key evaluation per snapshot element, followed by stable sorting.
//! Documentation: `docs/pascal/language/types/array/basics.md`.

use crate::{SharedArray, SourceLocation, Value};

use super::arguments::CallbackArguments;
use super::operation::{Advance, AdvanceError};

/// Snapshot and cached keys retained across task suspension.
pub(super) struct KeyedSort {
    values: SharedArray,
    entries: Vec<(Value, Value)>,
    location: SourceLocation,
}

impl KeyedSort {
    /// Retain the receiver snapshot before evaluating any keys.
    pub(super) fn new(values: SharedArray, location: SourceLocation) -> Self {
        Self {
            entries: Vec::with_capacity(values.len()),
            values,
            location,
        }
    }

    /// Save one key and request the next element or return the sorted result.
    pub(super) fn advance(&mut self, pending: Option<Value>) -> Result<Advance, AdvanceError> {
        if let Some(key) = pending {
            self.entries
                .push((key, self.values[self.entries.len()].clone()));
        }
        if let Some(value) = self.values.get(self.entries.len()) {
            Ok(Advance::Call(CallbackArguments::one(value.clone())))
        } else {
            crate::stdlib::sort_by_cached_keys(std::mem::take(&mut self.entries), self.location)
                .map(Advance::Complete)
                .map_err(AdvanceError::Sorting)
        }
    }
}

impl crate::TraceRoots for KeyedSort {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), crate::OwnershipError> {
        self.values.as_slice().trace_roots(visit)?;
        for (key, value) in &self.entries {
            visit(key);
            visit(value);
        }
        Ok(())
    }
}
