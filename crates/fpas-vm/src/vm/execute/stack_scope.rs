use super::super::diagnostics::VmError;
use super::super::{Worker, canonical_name, internal_error, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_UNDEFINED_GLOBAL;

impl Worker {
    pub(super) fn try_exec_stack_scope(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::Constant(idx) => {
                let val = self.const_value(idx, line)?.clone();
                self.push(val)?;
                Ok(true)
            }
            Op::Unit => {
                self.push(Value::Unit)?;
                Ok(true)
            }
            Op::Pop => {
                self.pop(line)?;
                Ok(true)
            }
            Op::Dup => {
                let val = self.peek(line)?.clone();
                self.push(val)?;
                Ok(true)
            }
            Op::GetLocal(slot) => {
                let idx = self.local_abs_index(0, slot, line)?;
                let val = self.stack[idx].clone();
                self.push(val)?;
                Ok(true)
            }
            Op::SetLocal(slot) => {
                let idx = self.local_abs_index(0, slot, line)?;
                let val = self.peek(line)?.clone();
                self.stack[idx] = val;
                Ok(true)
            }
            Op::GetGlobal(idx) => {
                let name = self.const_str(idx, line)?;
                let canonical = canonical_name(&name);
                let val = self
                    .shared
                    .globals
                    .read()
                    .unwrap_or_else(|e| e.into_inner())
                    .get(&canonical)
                    .cloned()
                    .ok_or_else(|| {
                        runtime_error(
                            RUNTIME_UNDEFINED_GLOBAL,
                            format!("Undefined global variable `{name}`"),
                            "Declare the global variable before reading it, or fix the variable name.",
                            line,
                        )
                    })?;
                self.push(val)?;
                Ok(true)
            }
            Op::SetGlobal(idx) => {
                let name = self.const_str(idx, line)?;
                let val = self.peek(line)?.clone();
                self.shared
                    .globals
                    .write()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(canonical_name(&name), val);
                Ok(true)
            }
            Op::GetEnclosing(depth, slot) => {
                let idx = self.local_abs_index(depth, slot, line)?;
                let val = self.stack[idx].clone();
                self.push(val)?;
                Ok(true)
            }
            Op::SetEnclosing(depth, slot) => {
                let val = self.stack.last().cloned().ok_or_else(|| {
                    internal_error(
                        "Stack underflow on SetEnclosing",
                        "This indicates invalid bytecode or a VM bug. Please report it.",
                        line,
                    )
                })?;
                let idx = self.local_abs_index(depth, slot, line)?;
                self.stack[idx] = val;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
