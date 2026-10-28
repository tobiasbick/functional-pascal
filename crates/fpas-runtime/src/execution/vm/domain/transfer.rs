//! Shared admission policy for values published across task boundaries.

use crate::{Value, classify_values, summarized_transferable};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use crate::execution::vm::worker::Worker;
use crate::execution::vm::{VmError, diagnostics};

/// Check complete graphs before retaining work, publishing messages or completing tasks.
///
/// Resource handles retain their existing registry rules. Executable provenance is checked
/// separately when importing values into a host session.
pub(in crate::execution::vm) fn check_transfer(values: &[Value]) -> Result<(), String> {
    if summarized_transferable(values) {
        return Ok(());
    }
    let graph = classify_values(values)
        .map_err(|error| format!("Cannot inspect value graph for task transfer: {error}"))?;
    if graph.is_transferable() {
        return Ok(());
    }
    let reason = if graph.has_references {
        "it contains a variable reference"
    } else if graph.task_bound {
        "it contains a task-bound callable"
    } else {
        "it contains mutable capture cells or local ownership"
    };
    Err(format!(
        "Cannot transfer value graph across a task boundary: {reason}"
    ))
}

impl Worker {
    pub(in crate::execution::vm) fn require_transferable_values(
        &self,
        values: &[Value],
    ) -> Result<(), VmError> {
        check_transfer(values).map_err(|message| {
            diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_INVALID_TASK,
                message,
                "Keep mutable captures and variable references on their owning task; pass immutable values or transferable callables.",
            )
        })
    }
}

#[cfg(test)]
mod tests;
