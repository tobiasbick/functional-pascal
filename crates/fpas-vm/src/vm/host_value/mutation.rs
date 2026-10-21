//! Guarded retained-value mutation, valid after execution retirement as well.

use super::{HostArgument, HostValue, HostValueError};
use fpas_bytecode::Value;
use std::sync::Arc;

impl HostValue {
    /// Read a captured cell into another registered, sealed value.
    ///
    /// # Errors
    /// Rejects non-cell values and unavailable or foreign storage.
    pub fn read_cell(&self) -> Result<Self, HostValueError> {
        let _access = self.0.origin.domain.access();
        let Value::Cell(cell) = &self.0.value else {
            return Err(HostValueError::NotCell);
        };
        let value = cell
            .read(&self.0.origin.domain)
            .map_err(HostValueError::Ownership)?;
        Ok(Self::new(value, Arc::clone(&self.0.origin)))
    }

    /// Replace a captured cell without invoking FPAS or reopening session resources.
    ///
    /// Existing aliases keep cell identity. Retired exports remain inspectable and
    /// mutable; execution re-entry still requires a live VM or callback session.
    ///
    /// # Errors
    /// Rejects non-cell values, foreign graphs, escaping references or foreign code.
    pub fn replace_cell(&self, replacement: HostArgument) -> Result<(), HostValueError> {
        // Validate foreign inputs under their own access before entering this domain.
        let value = self.mutation_input(&replacement)?;
        let _access = self.0.origin.domain.access();
        let Value::Cell(cell) = &self.0.value else {
            return Err(HostValueError::NotCell);
        };
        cell.replace(&self.0.origin.domain, value)
            .map_err(HostValueError::Ownership)?;
        Ok(())
    }

    /// Replace this retained root while preserving independent clones and session origin.
    ///
    /// # Errors
    /// Applies the same provenance and ownership checks as cell replacement.
    pub fn replace(&mut self, replacement: HostArgument) -> Result<(), HostValueError> {
        let value = self.mutation_input(&replacement)?;
        let origin = Arc::clone(&self.0.origin);
        let _access = origin.domain.access();
        *self = Self::new(value, Arc::clone(&origin));
        origin.domain.request_maintenance();
        Ok(())
    }

    fn mutation_input(&self, input: &HostArgument) -> Result<Value, HostValueError> {
        let (value, same_origin, _source) = match input {
            HostArgument::Data(value) => (value, false, None),
            HostArgument::Export(value) => (
                &value.0.value,
                Arc::ptr_eq(&self.0.origin, &value.0.origin),
                Some(value.0.origin.domain.access()),
            ),
        };
        let (graph, resources) = super::admission::inspect(value)?;
        if graph.has_references
            || graph
                .executable_identities
                .iter()
                .any(|identity| self.0.origin.executable.as_ref() != Some(identity))
        {
            return Err(HostValueError::SessionBound);
        }
        if same_origin {
            graph
                .validate_domain(&self.0.origin.domain)
                .map_err(HostValueError::Ownership)?;
        } else if !graph.is_transferable() || resources {
            return Err(HostValueError::SessionBound);
        }
        Ok(value.clone())
    }
}
