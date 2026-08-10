//! Identity-aware deep detachment of mutable debugger values.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, TryLockError};

use fpas_bytecode::{SharedEnum, SharedRecord, Value};

use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

pub(super) struct ValueDetacher {
    cells: HashMap<usize, Arc<Mutex<Value>>>,
    values: usize,
    maximum: usize,
}

impl ValueDetacher {
    pub(super) fn new(maximum: usize) -> Self {
        Self {
            cells: HashMap::new(),
            values: 0,
            maximum,
        }
    }

    pub(super) fn detach(&mut self, value: &Value) -> Result<Value, DebugSessionError> {
        self.values = self.values.saturating_add(1);
        if self.values > self.maximum {
            return Err(error(
                DebugErrorKind::CallLimit,
                format!(
                    "debug call detached-value count exceeds limit {}",
                    self.maximum
                ),
                "Use smaller arguments or inspect the aggregate without calling code.",
            ));
        }
        match value {
            Value::Integer(value) => Ok(Value::Integer(*value)),
            Value::Real(value) => Ok(Value::Real(*value)),
            Value::Boolean(value) => Ok(Value::Boolean(*value)),
            Value::Str(value) => Ok(Value::Str(value.clone())),
            Value::Unit => Ok(Value::Unit),
            Value::Array(values) => Ok(Value::Array(
                values
                    .iter()
                    .map(|value| self.detach(value))
                    .collect::<Result<Vec<_>, _>>()?
                    .into(),
            )),
            Value::Dict(entries) => Ok(Value::dict(
                entries
                    .iter()
                    .map(|(key, value)| Ok((self.detach(key)?, self.detach(value)?)))
                    .collect::<Result<Vec<_>, DebugSessionError>>()?,
            )),
            Value::Record(record) => Ok(Value::Record(SharedRecord::new(
                Arc::clone(&record.body().layout),
                record
                    .body()
                    .values
                    .iter()
                    .map(|value| self.detach(value))
                    .collect::<Result<Vec<_>, _>>()?,
            ))),
            Value::Enum(enumeration) => Ok(Value::Enum(SharedEnum::new(
                Arc::clone(&enumeration.body().layout),
                enumeration
                    .body()
                    .values
                    .iter()
                    .map(|value| self.detach(value))
                    .collect::<Result<Vec<_>, _>>()?,
            ))),
            Value::ResultOk(value) => Ok(Value::ResultOk(Box::new(self.detach(value)?))),
            Value::ResultError(value) => Ok(Value::ResultError(Box::new(self.detach(value)?))),
            Value::OptionSome(value) => Ok(Value::OptionSome(Box::new(self.detach(value)?))),
            Value::OptionNone => Ok(Value::OptionNone),
            Value::Function(function) => Ok(Value::function(
                function.function,
                function.name.clone(),
                function
                    .captures
                    .iter()
                    .map(|value| self.detach(value))
                    .collect::<Result<Vec<_>, _>>()?,
                false,
            )),
            Value::Cell(cell) => self.detach_cell(cell),
            Value::Task(_) => Err(error(
                DebugErrorKind::UnavailableValue,
                "debug calls cannot detach task handles",
                "Pass completed task results, not task handles, to debugger calls.",
            )),
            Value::OpaqueHandle(_) => Err(error(
                DebugErrorKind::UnavailableValue,
                "debug calls cannot detach opaque host handles",
                "Inspect the handle without passing it to debugger-side code.",
            )),
        }
    }

    fn detach_cell(&mut self, cell: &Arc<Mutex<Value>>) -> Result<Value, DebugSessionError> {
        let identity = Arc::as_ptr(cell) as usize;
        if let Some(detached) = self.cells.get(&identity) {
            return Ok(Value::Cell(Arc::clone(detached)));
        }
        let detached = Arc::new(Mutex::new(Value::Unit));
        self.cells.insert(identity, Arc::clone(&detached));
        let inner = match cell.try_lock() {
            Ok(value) => self.detach(&value)?,
            Err(TryLockError::WouldBlock) => {
                return Err(error(
                    DebugErrorKind::UnavailableValue,
                    "debug call value graph contains a busy mutable cell",
                    "Retry after the value is no longer contended.",
                ));
            }
            Err(TryLockError::Poisoned(_)) => {
                return Err(error(
                    DebugErrorKind::UnavailableValue,
                    "debug call value graph contains a poisoned mutable cell",
                    "Inspect the value without executing a call.",
                ));
            }
        };
        *detached
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = inner;
        Ok(Value::Cell(detached))
    }
}

pub(super) fn error(
    kind: DebugErrorKind,
    message: impl Into<String>,
    hint: impl Into<String>,
) -> DebugSessionError {
    DebugSessionError {
        kind,
        message: message.into(),
        hint: hint.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_cells_detach_once_without_aliasing_live_state() {
        let live = Arc::new(Mutex::new(Value::Integer(7)));
        let value = Value::Array(
            vec![
                Value::Cell(Arc::clone(&live)),
                Value::Cell(Arc::clone(&live)),
            ]
            .into(),
        );

        let Value::Array(detached) = ValueDetacher::new(16).detach(&value).expect("detach array")
        else {
            panic!("detached array expected");
        };
        let [Value::Cell(first), Value::Cell(second)] = detached.as_slice() else {
            panic!("detached cells expected");
        };

        assert!(Arc::ptr_eq(first, second));
        assert!(!Arc::ptr_eq(first, &live));
    }

    #[test]
    fn cyclic_cells_retain_the_cycle_inside_the_detached_graph() {
        let live = Arc::new(Mutex::new(Value::Unit));
        *live.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) =
            Value::Cell(Arc::clone(&live));

        let Value::Cell(detached) = ValueDetacher::new(16)
            .detach(&Value::Cell(Arc::clone(&live)))
            .expect("detach cycle")
        else {
            panic!("detached cell expected");
        };
        assert!(!Arc::ptr_eq(&detached, &live));
        let inner = detached
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Value::Cell(back_edge) = &*inner else {
            panic!("detached cycle back edge expected");
        };
        assert!(Arc::ptr_eq(&detached, back_edge));
    }
}
