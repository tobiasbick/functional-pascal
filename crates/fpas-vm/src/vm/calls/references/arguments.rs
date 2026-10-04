//! Parameter modes and activation at synchronous callable entry.

use std::sync::Arc;

use fpas_bytecode::{FunctionId, SelectedReference, Value};

use crate::vm::{VmError, worker::Worker};

impl Worker {
    /// Validate modes, same-root exclusion and provenance before activation.
    pub(in crate::vm) fn activate_reference_arguments<'a>(
        &self,
        target: FunctionId,
        arguments: impl IntoIterator<Item = &'a Value>,
        captures: &[Value],
    ) -> Result<Vec<Arc<SelectedReference>>, VmError> {
        let info = &self.executable.executable().functions[usize::from(target.get())];
        let mut references: Vec<Arc<SelectedReference>> = Vec::new();
        for (position, value) in arguments.into_iter().enumerate() {
            let is_var = info.var_parameters.binary_search(&(position as u8)).is_ok();
            match (is_var, value) {
                (true, Value::Reference(reference)) => {
                    if !reference.uses_registry(&self.hosted.references) {
                        return Err(
                            self.reference_transport_error("Reference belongs to another VM")
                        );
                    }
                    if references
                        .iter()
                        .any(|previous| previous.same_root(reference))
                    {
                        return Err(self.reference_transport_error(
                            "Two var arguments share one storage root",
                        ));
                    }
                    reference
                        .read()
                        .map_err(|error| self.reference_path_error(error))?;
                    references.push(Arc::clone(reference));
                }
                (true, _) => {
                    return Err(self
                        .reference_transport_error("Var parameter requires a reserved reference"));
                }
                (false, _) => self.require_value_data(value)?,
            }
        }
        for capture in captures {
            self.require_value_data(capture)?;
        }
        for (index, reference) in references.iter().enumerate() {
            if let Err(error) = reference.activate() {
                for activated in references[..index].iter().rev() {
                    let _ = activated.release();
                }
                return Err(self.reference_error(error));
            }
        }
        Ok(references)
    }

    /// Reject authority where retained or transported value data is required.
    pub(in crate::vm) fn require_value_data(&self, value: &Value) -> Result<(), VmError> {
        if value.contains_reference() {
            Err(self.reference_transport_error(
                "A var reference cannot be stored, captured, returned or transported",
            ))
        } else {
            Ok(())
        }
    }

    /// Report invalid authority transport with F4026 and the current source position.
    /// Documentation: `docs/pascal/tools/diagnostics.md`.
    pub(in crate::vm) fn reference_transport_error(&self, message: &str) -> VmError {
        crate::vm::diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            fpas_diagnostics::codes::RUNTIME_STORAGE_REFERENCE_CONFLICT,
            message,
            "Read the var parameter to obtain a value snapshot. Pass references only to synchronous var parameters.",
        )
    }
}
