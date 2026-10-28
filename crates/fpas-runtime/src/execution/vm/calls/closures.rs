//! Closure construction and mutable capture cells.

use super::*;

impl Worker {
    pub(in crate::execution::vm) fn make_closure(
        &mut self,
        operands: AbcOperands,
    ) -> Result<(), VmError> {
        let target = self.function_target(FunctionId::new(operands.b))?;
        let captures = self.clone_window(operands.c, operands.auxiliary)?;
        let classification = self.classify_local_values(&captures)?;
        // Direct nested helpers may borrow their enclosing `var` arguments.
        // Persistent storage and transfer boundaries enforce their non-escape rule.
        let task_bound = classification.task_bound || !classification.owners.is_empty();
        self.write(
            self.call_register(operands.a)?,
            if task_bound {
                Value::task_owned_function(target, captures, self.domain.id().clone())
            } else {
                Value::function(target, captures)
            },
        )
    }

    pub(in crate::execution::vm) fn make_cell(
        &mut self,
        operands: AbcOperands,
    ) -> Result<(), VmError> {
        let value = self.read(self.call_register(operands.b)?)?.clone();
        let cell = self
            .domain
            .new_cell(value)
            .map_err(|error| self.ownership_error(error))?;
        self.write(self.call_register(operands.a)?, Value::Cell(cell))
    }

    /// Copy a mutable capture only after checking the logical task's domain.
    pub(in crate::execution::vm) fn read_cell(
        &mut self,
        operands: AbcOperands,
    ) -> Result<(), VmError> {
        let mut value = Value::Unit;
        match self.read(self.call_register(operands.b)?)? {
            Value::Cell(cell) => {
                if let Some(value) = cell
                    .read_integer_or_value(&self.domain, &mut value)
                    .map_err(|error| self.ownership_error(error))?
                {
                    return self.write_integer_operand(operands.a, value);
                }
            }
            other => return Err(self.operand_type_error("cell", other)),
        }
        self.write(self.call_register(operands.a)?, value)
    }

    /// Validate the complete replacement before changing an owned capture cell.
    pub(in crate::execution::vm) fn write_cell(
        &mut self,
        operands: AbcOperands,
    ) -> Result<(), VmError> {
        let value = self.read(self.call_register(operands.b)?)?;
        match self.read(self.call_register(operands.a)?)? {
            Value::Cell(cell) => match value {
                Value::Integer(value) => cell.replace_integer(&self.domain, *value),
                value => cell.replace(&self.domain, value.clone()),
            }
            .map_err(|error| self.ownership_error(error)),
            other => Err(self.operand_type_error("cell", other)),
        }
    }
}
