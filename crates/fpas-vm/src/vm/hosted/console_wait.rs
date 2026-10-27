//! Cooperative timed terminal reads for the debugger's queue-only input.
//!
//! Documentation: `docs/pascal/std/console/events.md`.

use std::time::Duration;

use fpas_bytecode::{Register, SourceLocation, Value};

use super::console_args::{integer, optional_event};
use crate::vm::{VmError, tasks::TaskSuspension, worker::Worker};

impl Worker {
    /// Read queued input immediately, or retain the destination until input or timeout.
    pub(in crate::vm) fn debug_console_wait(
        &mut self,
        arguments: &[Value],
        destination: Option<Register>,
        location: SourceLocation,
    ) -> Result<Option<Value>, VmError> {
        let milliseconds = integer(arguments, 0, 1, self)?;
        let event = self
            .hosted
            .key_input
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .read_event_timeout(0, location)?;
        if event.is_some() || milliseconds <= 0 {
            return Ok(Some(optional_event(self, event, location)?));
        }
        self.task_suspension = Some(TaskSuspension::ConsoleRead {
            deadline_millis: self
                .task_clock_ref()
                .deadline_after(Duration::from_millis(milliseconds as u64)),
            destination,
            location,
        });
        self.suspend_requested = true;
        Ok(None)
    }

    /// Consume input only on the VM thread, preserving queued event precedence at expiry.
    pub(in crate::vm) fn poll_console_wait(
        &mut self,
        deadline_millis: u64,
        destination: Option<Register>,
        location: SourceLocation,
    ) -> Result<bool, VmError> {
        let event = self
            .hosted
            .key_input
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .read_event_timeout(0, location)?;
        if event.is_none() && self.task_clock_ref().now_millis() < deadline_millis {
            self.task_suspension = Some(TaskSuspension::ConsoleRead {
                deadline_millis,
                destination,
                location,
            });
            return Ok(false);
        }
        if let Some(destination) = destination {
            self.write(destination, optional_event(self, event, location)?)?;
        }
        Ok(true)
    }
}
