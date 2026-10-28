//! Run-local state management for source-defined HTTP registries.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::{HttpIntrinsic, Intrinsic, SourceLocation, Value, release_values};
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_INTRINSIC_STACK_STATE_ERROR,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::VmError;
use super::super::worker::Worker;

#[derive(Default)]
struct HttpStates {
    next_body_stream: i64,
    body_streams: HashMap<i64, Value>,
    next_sse_decoder: i64,
    sse_decoders: HashMap<i64, Value>,
}

/// Synchronized HTTP state shared by all tasks in one VM run.
pub(super) struct HttpStateRegistry {
    states: Mutex<HttpStates>,
}

impl HttpStateRegistry {
    /// Creates empty one-based registries for both HTTP handle kinds.
    pub(super) fn new() -> Self {
        Self {
            states: Mutex::new(HttpStates {
                next_body_stream: 1,
                next_sse_decoder: 1,
                ..HttpStates::default()
            }),
        }
    }

    fn reserve(&self, operation: HttpIntrinsic, state: Value) -> Result<Option<i64>, String> {
        if let Err(message) =
            crate::execution::vm::domain::check_transfer(std::slice::from_ref(&state))
        {
            release_values(vec![state]);
            return Err(message);
        }
        let mut states = self
            .states
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let slot = match operation {
            HttpIntrinsic::ReserveBodyStreamState => {
                let slot = states.next_body_stream;
                let Some(next) = slot.checked_add(1) else {
                    drop(states);
                    release_values(vec![state]);
                    return Ok(None);
                };
                states.next_body_stream = next;
                states.body_streams.insert(slot, state);
                slot
            }
            HttpIntrinsic::ReserveSseDecoderState => {
                let slot = states.next_sse_decoder;
                let Some(next) = slot.checked_add(1) else {
                    drop(states);
                    release_values(vec![state]);
                    return Ok(None);
                };
                states.next_sse_decoder = next;
                states.sse_decoders.insert(slot, state);
                slot
            }
            _ => {
                drop(states);
                release_values(vec![state]);
                return Ok(None);
            }
        };
        Ok(Some(slot))
    }

    fn contains(&self, operation: HttpIntrinsic, slot: i64) -> bool {
        let states = self
            .states
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match operation {
            HttpIntrinsic::HasBodyStreamState => states.body_streams.contains_key(&slot),
            HttpIntrinsic::HasSseDecoderState => states.sse_decoders.contains_key(&slot),
            _ => false,
        }
    }

    fn load(&self, operation: HttpIntrinsic, slot: i64) -> Option<Value> {
        let states = self
            .states
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match operation {
            HttpIntrinsic::LoadBodyStreamState => states.body_streams.get(&slot).cloned(),
            HttpIntrinsic::LoadSseDecoderState => states.sse_decoders.get(&slot).cloned(),
            _ => None,
        }
    }

    fn store(&self, operation: HttpIntrinsic, slot: i64, state: Value) -> Result<bool, String> {
        if let Err(message) =
            crate::execution::vm::domain::check_transfer(std::slice::from_ref(&state))
        {
            release_values(vec![state]);
            return Err(message);
        }
        let mut states = self
            .states
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let registry = match operation {
            HttpIntrinsic::StoreBodyStreamState => &mut states.body_streams,
            HttpIntrinsic::StoreSseDecoderState => &mut states.sse_decoders,
            _ => {
                drop(states);
                release_values(vec![state]);
                return Ok(false);
            }
        };
        let Some(current) = registry.get_mut(&slot) else {
            drop(states);
            release_values(vec![state]);
            return Ok(false);
        };
        let previous = std::mem::replace(current, state);
        drop(states);
        release_values(vec![previous]);
        Ok(true)
    }
}

impl Worker {
    pub(super) fn execute_http_state_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        _location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Http(operation) = intrinsic else {
            return Ok(None);
        };
        let value = match operation {
            HttpIntrinsic::ReserveBodyStreamState | HttpIntrinsic::ReserveSseDecoderState => {
                require_count(self, arguments, 1)?;
                let state = record(self, &arguments[0])?.clone();
                let slot = self
                    .hosted
                    .http_states
                    .reserve(operation, state)
                    .map_err(|message| self.invalid_local_value(message))?
                    .ok_or_else(|| {
                        self.runtime_error(
                            RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                            "HTTP state registry exhausted the FPAS integer range",
                            "Start a new program run before allocating more HTTP handles.",
                        )
                    })?;
                Some(Value::Integer(slot))
            }
            HttpIntrinsic::HasBodyStreamState | HttpIntrinsic::HasSseDecoderState => {
                require_count(self, arguments, 1)?;
                let slot = integer(self, &arguments[0])?;
                Some(Value::Boolean(
                    self.hosted.http_states.contains(operation, slot),
                ))
            }
            HttpIntrinsic::LoadBodyStreamState | HttpIntrinsic::LoadSseDecoderState => {
                require_count(self, arguments, 1)?;
                let slot = integer(self, &arguments[0])?;
                Some(
                    self.hosted
                        .http_states
                        .load(operation, slot)
                        .ok_or_else(|| {
                            self.runtime_error(
                                RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                                format!("HTTP state slot {slot} is invalid"),
                                "Use a handle returned by the matching Std.Net.Http creation function.",
                            )
                        })?,
                )
            }
            HttpIntrinsic::StoreBodyStreamState | HttpIntrinsic::StoreSseDecoderState => {
                require_count(self, arguments, 2)?;
                let slot = integer(self, &arguments[0])?;
                let state = record(self, &arguments[1])?.clone();
                if !self
                    .hosted
                    .http_states
                    .store(operation, slot, state)
                    .map_err(|message| self.invalid_local_value(message))?
                {
                    return Err(self.runtime_error(
                        RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                        format!("HTTP state slot {slot} is invalid"),
                        "Use a handle returned by the matching Std.Net.Http creation function.",
                    ));
                }
                None
            }
        };
        Ok(Some(value))
    }
}

fn require_count(worker: &Worker, arguments: &[Value], expected: usize) -> Result<(), VmError> {
    if arguments.len() == expected {
        return Ok(());
    }
    Err(worker.runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!(
            "Std.Net.Http state intrinsic expected {expected} arguments, got {}",
            arguments.len()
        ),
        "Check the compiler intrinsic signature and register argument count.",
    ))
}

fn integer(worker: &Worker, value: &Value) -> Result<i64, VmError> {
    match value {
        Value::Integer(value) => Ok(*value),
        actual => Err(type_error(worker, "integer", actual)),
    }
}

fn record<'a>(worker: &Worker, value: &'a Value) -> Result<&'a Value, VmError> {
    match value {
        Value::Record(_) => Ok(value),
        actual => Err(type_error(worker, "record", actual)),
    }
}

fn type_error(worker: &Worker, expected: &str, actual: &Value) -> VmError {
    worker.runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!(
            "Std.Net.Http state intrinsic expected {expected}, got {}",
            actual.type_name()
        ),
        "Pass values matching the compiler-provided internal Std.Net.Http signature.",
    )
}

#[cfg(test)]
mod ownership_tests;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;

    use super::*;

    fn state(value: i64) -> Value {
        Value::Integer(value)
    }

    fn assert_concurrent_reservations_are_unique(reserve: HttpIntrinsic, load: HttpIntrinsic) {
        let registry = Arc::new(HttpStateRegistry::new());
        let workers: Vec<_> = (0..64)
            .map(|value| {
                let registry = Arc::clone(&registry);
                thread::spawn(move || registry.reserve(reserve, state(value)))
            })
            .collect();

        let slots: Vec<_> = workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .expect("state worker should complete")
                    .expect("state should be transferable")
                    .expect("slot range should remain available")
            })
            .collect();

        assert_eq!(
            slots.iter().collect::<std::collections::HashSet<_>>().len(),
            64
        );
        assert!(
            slots
                .into_iter()
                .all(|slot| registry.load(load, slot).is_some())
        );
    }

    #[test]
    fn concurrent_body_stream_reservations_retain_every_state() {
        assert_concurrent_reservations_are_unique(
            HttpIntrinsic::ReserveBodyStreamState,
            HttpIntrinsic::LoadBodyStreamState,
        );
    }

    #[test]
    fn concurrent_sse_reservations_retain_every_state() {
        assert_concurrent_reservations_are_unique(
            HttpIntrinsic::ReserveSseDecoderState,
            HttpIntrinsic::LoadSseDecoderState,
        );
    }
}
