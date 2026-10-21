//! Runtime graph ownership checks shared by execution and mutation boundaries.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

mod collector;
mod execution;
mod native;
#[cfg(test)]
mod protocol_tests;
#[cfg(test)]
mod root_tests;
pub(in crate::vm) mod roots;
mod transfer;

pub(in crate::vm) use collector::{configure, new_domain, new_session_domain, related_domain};
pub(in crate::vm) use transfer::check_transfer;

use fpas_bytecode::{GraphClassification, OwnershipError, Value, classify_values, validate_local};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use super::worker::Worker;
use super::{VmError, diagnostics};

impl Worker {
    pub(in crate::vm) fn ownership_error(&self, error: OwnershipError) -> VmError {
        self.invalid_local_value(error.to_string())
    }

    pub(in crate::vm) fn invalid_local_value(&self, message: impl Into<String>) -> VmError {
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            RUNTIME_INVALID_TASK,
            message,
            "Access mutable captures only from their owning task or session.",
        )
    }

    /// Check the complete contained graph before replacing owned storage.
    pub(in crate::vm) fn require_local_value(&self, value: &Value) -> Result<(), VmError> {
        validate_local(&self.domain, value).map_err(|error| self.ownership_error(error))
    }

    /// Persistent storage cannot retain an ephemeral `var` reference.
    pub(in crate::vm) fn require_storable_value(&self, value: &Value) -> Result<(), VmError> {
        if matches!(
            value,
            Value::Integer(_)
                | Value::Real(_)
                | Value::Boolean(_)
                | Value::Str(_)
                | Value::Unit
                | Value::OptionNone
                | Value::Task(_)
                | Value::OpaqueHandle(_)
        ) {
            return Ok(());
        }
        let classification = self.classify_local_values(std::slice::from_ref(value))?;
        if classification.has_references {
            return Err(
                self.invalid_local_value("A variable reference cannot escape into stored values")
            );
        }
        Ok(())
    }

    /// Classify all captures together so shared subgraphs are visited once.
    pub(in crate::vm) fn classify_local_values(
        &self,
        values: &[Value],
    ) -> Result<GraphClassification, VmError> {
        let classification =
            classify_values(values).map_err(|error| self.ownership_error(error))?;
        if classification
            .owners
            .iter()
            .any(|owner| owner != self.domain.id())
        {
            return Err(
                self.invalid_local_value("Value graph belongs to a foreign task or session")
            );
        }
        Ok(classification)
    }
}
