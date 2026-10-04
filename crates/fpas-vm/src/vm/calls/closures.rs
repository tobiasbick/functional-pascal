//! Closure construction and mutable capture cells.

use super::*;

impl Worker {
    pub(in crate::vm) fn make_closure(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let target = FunctionId::new(operands.b);
        let captures = self.clone_window(operands.c, operands.auxiliary)?;
        let task_bound = captures.iter().any(|capture| match capture {
            Value::Cell(_) => true,
            Value::Function(function) => function.task_bound,
            _ => false,
        });
        let name = self.function_name(target)?;
        self.write(
            self.call_register(operands.a)?,
            if task_bound {
                Value::task_owned_function(target, name, captures, self.task_id)
            } else {
                Value::function(target, name, captures)
            },
        )
    }

    pub(in crate::vm) fn make_cell(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let value = self.read(self.call_register(operands.b)?)?.clone();
        self.require_value_data(&value)?;
        self.write(
            self.call_register(operands.a)?,
            Value::Cell(Arc::new(Mutex::new(value))),
        )
    }

    /// Copy the value held by a mutable capture cell.
    ///
    /// The cell is borrowed from its register rather than cloned. Its storage-root
    /// authority is checked before taking the value snapshot.
    pub(in crate::vm) fn read_cell(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let value = match self.read(self.call_register(operands.b)?)? {
            Value::Cell(cell) => self
                .hosted
                .references
                .read(cell)
                .map_err(|error| self.reference_error(error))?,
            other => return Err(self.operand_type_error("cell", other)),
        };
        self.write(self.call_register(operands.a)?, value)
    }

    /// Replace the value held by a mutable capture cell.
    pub(in crate::vm) fn write_cell(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let value = self.read(self.call_register(operands.b)?)?.clone();
        self.require_value_data(&value)?;
        let storage = self.read(self.call_register(operands.a)?)?.clone();
        match &storage {
            Value::Cell(cell) => self
                .hosted
                .references
                .write(cell, value)
                .map_err(|error| self.reference_error(error))?,
            other => return Err(self.operand_type_error("cell", other)),
        }
        self.note_debug_storage_store(&storage);
        Ok(())
    }

    /// Report a storage-authority conflict at the current instruction's source position.
    /// Documentation: `docs/pascal/tools/diagnostics.md`.
    pub(in crate::vm) fn reference_error(&self, error: fpas_bytecode::ReferenceError) -> VmError {
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            fpas_diagnostics::codes::RUNTIME_STORAGE_REFERENCE_CONFLICT,
            error.to_string(),
            "Use the authorized var parameter inside the call. Do not access its storage root through another alias until the call returns.",
        )
    }
}
