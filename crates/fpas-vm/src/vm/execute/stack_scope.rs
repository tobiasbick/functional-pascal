use super::super::{Vm, VmError, internal_error, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_UNDEFINED_GLOBAL;

impl Vm {
    pub(super) fn try_exec_stack_scope(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::Constant(idx) => {
                let val = self.chunk.constants[idx as usize].clone();
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
                let base = self.frame_base();
                let val = self.stack[base + slot as usize].clone();
                self.push(val)?;
                Ok(true)
            }
            Op::SetLocal(slot) => {
                let base = self.frame_base();
                let val = self.peek(line)?.clone();
                self.stack[base + slot as usize] = val;
                Ok(true)
            }
            Op::GetGlobal(idx) => {
                let name = self.const_str(idx, line)?;
                let val = self.globals.get(&name).cloned().ok_or_else(|| {
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
                self.globals.insert(name, val);
                Ok(true)
            }
            Op::GetEnclosing(depth, slot) => {
                let cs_len = self.call_stack.len();
                let base = if (depth as usize) >= cs_len {
                    0
                } else {
                    self.call_stack[cs_len - 1 - depth as usize].base_slot
                };
                let val = self.stack[base + slot as usize].clone();
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
                let cs_len = self.call_stack.len();
                let base = if (depth as usize) >= cs_len {
                    0
                } else {
                    self.call_stack[cs_len - 1 - depth as usize].base_slot
                };
                self.stack[base + slot as usize] = val;
                Ok(true)
            }
            Op::MakeClosure(num_captures) => {
                let func = self.pop(line)?;
                let mut captures = Vec::with_capacity(num_captures as usize);
                for _ in 0..num_captures {
                    captures.push(self.pop(line)?);
                }
                captures.reverse();
                match func {
                    Value::Function { name, .. } => {
                        self.push(Value::Function { name, captures })?;
                    }
                    _ => {
                        return Err(internal_error(
                            "MakeClosure expected a Function value on stack",
                            "This indicates a compiler bug. Please report it.",
                            line,
                        ));
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
