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
        self.write(
            self.call_register(operands.a)?,
            Value::Cell(Arc::new(Mutex::new(value))),
        )
    }

    /// Copy the value held by a mutable capture cell.
    ///
    /// The cell is borrowed from its register rather than cloned, so an access costs one
    /// uncontended lock instead of an extra reference-count round trip.
    pub(in crate::vm) fn read_cell(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let value = match self.read(self.call_register(operands.b)?)? {
            Value::Cell(cell) => cell
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone(),
            other => return Err(self.operand_type_error("cell", other)),
        };
        self.write(self.call_register(operands.a)?, value)
    }

    /// Replace the value held by a mutable capture cell.
    pub(in crate::vm) fn write_cell(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let value = self.read(self.call_register(operands.b)?)?.clone();
        match self.read(self.call_register(operands.a)?)? {
            Value::Cell(cell) => {
                *cell.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = value;
                Ok(())
            }
            other => Err(self.operand_type_error("cell", other)),
        }
    }
}
