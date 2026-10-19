//! Bounded live declaration instances; completed calls retain events only.

use super::{TraceHistory, model::MAX_SCALAR_BYTES};
use fpas_bytecode::{Executable, Value};

pub(super) const MAX_LOCAL_INSTANCES: usize = 1024;
pub(super) const MAX_LOCAL_BYTES: usize = 8 * 1024 * 1024;

pub(super) struct LocalLast {
    watch: usize,
    activation: u64,
    value: Option<Value>,
}

impl TraceHistory {
    pub(super) fn activation(&mut self) -> Option<u64> {
        self.next_activation = self.next_activation.checked_add(1).or_else(|| {
            self.fail("trace_value_limit", false);
            None
        })?;
        Some(self.next_activation)
    }

    pub(super) fn instance(&mut self, watch: usize, activation: u64) -> Option<u64> {
        if self.local_values.len() >= MAX_LOCAL_INSTANCES {
            self.fail("trace_value_limit", false);
            return None;
        }
        self.next_instance = self.next_instance.checked_add(1).or_else(|| {
            self.fail("trace_value_limit", false);
            None
        })?;
        self.local_values.insert(
            self.next_instance,
            LocalLast {
                watch,
                activation,
                value: None,
            },
        );
        Some(self.next_instance)
    }

    pub(super) fn retire(&mut self, instance: u64) {
        if let Some(last) = self.local_values.remove(&instance) {
            self.local_bytes = self.local_bytes.saturating_sub(bytes(last.value.as_ref()));
        }
    }

    pub(super) fn observe_local(
        &mut self,
        image: &Executable,
        instance: u64,
        value: Value,
        site: super::pending::TraceSite<'_>,
    ) {
        if self.limit.is_some() {
            return;
        }
        let Some(last) = self.local_values.get(&instance) else {
            return;
        };
        if !valid(&value) {
            self.fail("trace_value_limit", false);
            return;
        }
        let old = last.value.clone();
        // Immutable identical string storage requires neither scanning nor comparison budget.
        if let (Some(Value::Str(left)), Value::Str(right)) = (&old, &value)
            && left.as_ptr() == right.as_ptr()
            && left.len() == right.len()
        {
            return;
        }
        let watch = last.watch;
        let activation = last.activation;
        let retained = self
            .local_bytes
            .saturating_sub(bytes(old.as_ref()))
            .saturating_add(bytes(Some(&value)));
        if retained > MAX_LOCAL_BYTES {
            self.fail("trace_value_limit", false);
            return;
        }
        self.local_bytes = retained;
        if let Some(last) = self.local_values.get_mut(&instance) {
            last.value = Some(value.clone());
        }
        self.commit(
            image,
            vec![(watch, old)],
            vec![(watch, Some(value))],
            site,
            Some((activation, instance)),
        );
    }
}

fn bytes(value: Option<&Value>) -> usize {
    match value {
        Some(Value::Str(text)) => text.len(),
        _ => 0,
    }
}

fn valid(value: &Value) -> bool {
    match value {
        Value::Integer(_) | Value::Boolean(_) => true,
        Value::Real(value) => value.is_finite(),
        Value::Str(text) => text.len() <= MAX_SCALAR_BYTES,
        Value::Enum(value) => {
            value.body().values.is_empty()
                && value.body().layout.type_name.len() <= 4096
                && value.body().layout.variant.len() <= 4096
        }
        _ => false,
    }
}
