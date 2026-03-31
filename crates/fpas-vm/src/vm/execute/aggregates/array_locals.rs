use super::super::super::diagnostics::VmError;
use super::super::super::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_POP_FROM_EMPTY_ARRAY, RUNTIME_VM_OPERAND_TYPE_MISMATCH};

impl Worker {
    pub(super) fn exec_array_push_local(
        &mut self,
        depth: u16,
        slot: u16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let value = self.pop(line)?;
        let idx = self.local_abs_index(depth, slot, line)?;
        match &mut self.stack[idx] {
            Value::Array(elems) => elems.push(value),
            other => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "ArrayPushLocal: local is {}, expected array",
                        other.type_name()
                    ),
                    "Pass a mutable local array variable to Std.Array.Push.",
                    line,
                ));
            }
        }
        Ok(())
    }

    pub(super) fn exec_array_pop_local(
        &mut self,
        depth: u16,
        slot: u16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let idx = self.local_abs_index(depth, slot, line)?;
        let Value::Array(elems) = &mut self.stack[idx] else {
            return Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "ArrayPopLocal: local is {}, expected array",
                    self.stack[idx].type_name()
                ),
                "Pass a mutable local array variable to Std.Array.Pop.",
                line,
            ));
        };
        let popped = elems.pop().ok_or_else(|| {
            runtime_error(
                RUNTIME_POP_FROM_EMPTY_ARRAY,
                "Pop from empty array",
                "Check array length before popping an element.",
                line,
            )
        })?;
        self.push(popped)?;
        Ok(())
    }
}
