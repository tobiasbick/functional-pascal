//! Checked reservation, path projection and immediate selected storage operations.

use std::sync::Arc;

use fpas_bytecode::{
    AbcOperands, RecordTypeId, ReferencePathError, ReferenceStep, SelectedReference, Value,
};

use crate::vm::{VmError, worker::Worker};

impl Worker {
    /// Reserve a root or forward an active parameter before later argument evaluation.
    pub(in crate::vm) fn reserve_reference(
        &mut self,
        operands: AbcOperands,
    ) -> Result<(), VmError> {
        let reference = match self.read_operand(operands.b)? {
            Value::Cell(cell) => SelectedReference::root(
                self.hosted
                    .references
                    .reserve(cell)
                    .map_err(|error| self.reference_error(error))?,
            ),
            Value::Reference(reference) => reference
                .reborrow()
                .map_err(|error| self.reference_error(error))?,
            other => return Err(self.operand_type_error("cell or var reference", other)),
        };
        let reference = Arc::new(reference);
        self.reference_scopes.reserve(Arc::clone(&reference));
        self.write_operand(operands.a, Value::Reference(reference))
    }

    /// Retain a checked record field under the existing reservation.
    pub(in crate::vm) fn select_reference_field(
        &mut self,
        operands: AbcOperands,
    ) -> Result<(), VmError> {
        let reference = self
            .reference_operand(operands.a)?
            .project(ReferenceStep::Field {
                record: RecordTypeId::new(operands.b),
                field: usize::from(operands.c),
            })
            .map_err(|error| self.reference_path_error(error))?;
        self.write_operand(operands.a, Value::Reference(Arc::new(reference)))
    }

    /// Freeze an evaluated index and validate the selected existing element.
    pub(in crate::vm) fn select_reference_index(
        &mut self,
        operands: AbcOperands,
    ) -> Result<(), VmError> {
        let index = self.read_operand(operands.c)?.clone();
        self.require_value_data(&index)?;
        let reference = self
            .reference_operand(operands.b)?
            .project(ReferenceStep::Index(index))
            .map_err(|error| self.reference_path_error(error))?;
        self.write_operand(operands.a, Value::Reference(Arc::new(reference)))
    }

    /// Copy the selected value through its current authority.
    pub(in crate::vm) fn read_reference(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let value = self
            .reference_operand(operands.b)?
            .read()
            .map_err(|error| self.reference_path_error(error))?;
        self.write_operand(operands.a, value)
    }

    /// Immediately replace selected value data without retaining another reference.
    pub(in crate::vm) fn write_reference(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let value = self.read_operand(operands.b)?.clone();
        self.require_value_data(&value)?;
        let storage = self.read_operand(operands.a)?.clone();
        self.reference_operand(operands.a)?
            .write(value)
            .map_err(|error| self.reference_path_error(error))?;
        self.note_debug_storage_store(&storage);
        Ok(())
    }

    /// Invalidate every copy of the completed selected reference.
    pub(in crate::vm) fn release_reference(
        &mut self,
        operands: AbcOperands,
    ) -> Result<(), VmError> {
        let reference = self.reference_operand(operands.a)?.clone();
        reference
            .release()
            .map_err(|error| self.reference_error(error))?;
        self.reference_scopes.forget(&reference);
        Ok(())
    }

    fn reference_operand(&self, register: u16) -> Result<&SelectedReference, VmError> {
        match self.read_operand(register)? {
            Value::Reference(reference) => Ok(reference),
            other => Err(self.operand_type_error("var reference", other)),
        }
    }

    /// Preserve bounds/key errors and report unauthorized access as F4026.
    /// Documentation: `docs/pascal/tools/diagnostics.md`.
    pub(in crate::vm) fn reference_path_error(&self, error: ReferencePathError) -> VmError {
        let code = match &error {
            ReferencePathError::Access(_) => {
                fpas_diagnostics::codes::RUNTIME_STORAGE_REFERENCE_CONFLICT
            }
            ReferencePathError::ArrayBounds { .. } => {
                fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS
            }
            ReferencePathError::MissingKey(_) => {
                fpas_diagnostics::codes::RUNTIME_DICT_KEY_NOT_FOUND
            }
            _ => fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        };
        crate::vm::diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            code,
            error.to_string(),
            "Pass an existing writable field or collection element through its authorized var reference.",
        )
    }
}
