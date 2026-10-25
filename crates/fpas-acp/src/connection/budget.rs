//! Bounded retained SDK metadata, released exactly once through ordinary ownership.

use super::Fault;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub(super) struct Budget {
    used: AtomicUsize,
    maximum: usize,
    name: &'static str,
}
pub(super) struct Permit {
    budget: Arc<Budget>,
    bytes: usize,
}

impl Budget {
    pub fn new(name: &'static str, maximum: usize) -> Arc<Self> {
        Arc::new(Self {
            used: AtomicUsize::new(0),
            maximum,
            name,
        })
    }
    pub fn reserve(self: &Arc<Self>, bytes: usize) -> Result<Permit, Fault> {
        self.used
            .try_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes)
                    .filter(|total| *total <= self.maximum)
            })
            .map_err(|_| Fault::LimitExceeded(self.name, self.maximum))?;
        Ok(Permit {
            budget: self.clone(),
            bytes,
        })
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.budget.used.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

pub(super) fn json_size(value: &serde_json::Value) -> usize {
    std::mem::size_of::<serde_json::Value>().saturating_add(match value {
        serde_json::Value::String(text) => text.len(),
        serde_json::Value::Number(number) => number.to_string().len(),
        serde_json::Value::Array(values) => values.iter().map(json_size).sum(),
        serde_json::Value::Object(values) => values
            .iter()
            .map(|(key, value)| key.len() + 64 + json_size(value))
            .sum(),
        _ => 0,
    })
}
