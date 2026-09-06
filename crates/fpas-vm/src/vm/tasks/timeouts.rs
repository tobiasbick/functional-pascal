//! Duration validation shared by task and channel waits.

use crate::vm::{VmError, worker::Worker};
use fpas_bytecode::Value;
use std::time::Duration;

impl Worker {
    /// Validate a non-negative millisecond duration for hosted task waits.
    pub(in crate::vm::tasks) fn wait_timeout(&self, value: &Value) -> Result<Duration, VmError> {
        let Value::Integer(milliseconds) = value else {
            return Err(self.task_type_error("non-negative timeout in milliseconds", value));
        };
        let milliseconds = u64::try_from(*milliseconds)
            .map_err(|_| self.task_type_error("non-negative timeout in milliseconds", value))?;
        Ok(Duration::from_millis(milliseconds))
    }
}
