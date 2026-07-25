use crate::vm::diagnostics::VmError;
use crate::vm::{CallFrame, Worker, internal_error, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_UNDEFINED_FUNCTION, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};

impl Worker {
    pub(super) fn try_exec_control_calls(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::Jump(addr) => {
                let target = addr as usize;
                if target >= self.shared.chunk.code().len() {
                    return Err(internal_error(
                        format!(
                            "Jump target {target} is out of bounds (code len {})",
                            self.shared.chunk.code().len()
                        ),
                        "This indicates malformed bytecode or a compiler control-flow bug. Please report it.",
                        line,
                    ));
                }
                self.ip = target;
                Ok(true)
            }
            Op::JumpIfFalse(addr) => {
                let val = self.pop(line)?;
                if !self.is_truthy(&val) {
                    let target = addr as usize;
                    if target >= self.shared.chunk.code().len() {
                        return Err(internal_error(
                            format!(
                                "JumpIfFalse target {target} is out of bounds (code len {})",
                                self.shared.chunk.code().len()
                            ),
                            "This indicates malformed bytecode or a compiler control-flow bug. Please report it.",
                            line,
                        ));
                    }
                    self.ip = target;
                }
                Ok(true)
            }
            Op::JumpIfTrue(addr) => {
                let val = self.pop(line)?;
                if self.is_truthy(&val) {
                    let target = addr as usize;
                    if target >= self.shared.chunk.code().len() {
                        return Err(internal_error(
                            format!(
                                "JumpIfTrue target {target} is out of bounds (code len {})",
                                self.shared.chunk.code().len()
                            ),
                            "This indicates malformed bytecode or a compiler control-flow bug. Please report it.",
                            line,
                        ));
                    }
                    self.ip = target;
                }
                Ok(true)
            }
            Op::JumpIfLocalGt(a, b, addr) => {
                self.jump_if_local_cmp(a, b, addr, line, |left, right| left > right)
            }
            Op::JumpIfLocalLt(a, b, addr) => {
                self.jump_if_local_cmp(a, b, addr, line, |left, right| left < right)
            }
            Op::Call(name_idx, argc) => {
                let (code_start, base_slot) = {
                    let name = self.const_str_ref(name_idx, line)?;
                    self.resolve_named_call(name, argc, line)
                }?;
                self.enter_function(code_start, base_slot, line)?;
                Ok(true)
            }
            Op::CallValue(argc) => {
                let func = self.pop(line)?;
                let function = match func {
                    Value::Function(function) => function,
                    other => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            format!("Expected function value, got `{}`", other.type_name()),
                            "Only function values can be called with CallValue. Check that the variable holds a function.",
                            line,
                        ));
                    }
                };
                self.call_named_function(&function.name, argc, line)?;
                // After `call_named_function`, `base_slot` points at the first argument. Pushing
                // captures on top matches the compiler layout: parameters then closure cells as
                // successive locals for the callee.
                for capture in &function.captures {
                    self.push(capture.clone())?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn jump_if_local_cmp(
        &mut self,
        slot_a: u16,
        slot_b: u16,
        addr: u32,
        line: SourceLocation,
        pred: impl FnOnce(i64, i64) -> bool,
    ) -> Result<bool, VmError> {
        let idx_a = self.local_abs_index(0, slot_a, line)?;
        let idx_b = self.local_abs_index(0, slot_b, line)?;
        let left = match self.stack.get(idx_a) {
            Some(Value::Integer(n)) => *n,
            Some(other) => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!("local compare expects integer, got {}", other.type_name()),
                    "Use JumpIfLocal* only on integer locals.",
                    line,
                ));
            }
            None => {
                return Err(internal_error(
                    "local compare index out of range",
                    "This indicates invalid bytecode or a VM bug. Please report it.",
                    line,
                ));
            }
        };
        let right = match self.stack.get(idx_b) {
            Some(Value::Integer(n)) => *n,
            Some(other) => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!("local compare expects integer, got {}", other.type_name()),
                    "Use JumpIfLocal* only on integer locals.",
                    line,
                ));
            }
            None => {
                return Err(internal_error(
                    "local compare index out of range",
                    "This indicates invalid bytecode or a VM bug. Please report it.",
                    line,
                ));
            }
        };
        if pred(left, right) {
            let target = addr as usize;
            if target >= self.shared.chunk.code().len() {
                return Err(internal_error(
                    format!(
                        "JumpIfLocal* target {target} is out of bounds (code len {})",
                        self.shared.chunk.code().len()
                    ),
                    "This indicates malformed bytecode or a compiler control-flow bug. Please report it.",
                    line,
                ));
            }
            self.ip = target;
        }
        Ok(true)
    }

    fn call_named_function(
        &mut self,
        name: &str,
        argc: u8,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let (code_start, base_slot) = self.resolve_named_call(name, argc, line)?;
        self.enter_function(code_start, base_slot, line)
    }

    fn resolve_named_call(
        &self,
        name: &str,
        argc: u8,
        line: SourceLocation,
    ) -> Result<(usize, usize), VmError> {
        let (code_start, expected_arity) = self.lookup_function_entry(name).ok_or_else(|| {
            runtime_error(
                RUNTIME_UNDEFINED_FUNCTION,
                format!("Undefined function `{name}`"),
                "Declare the function before calling it, or fix the function name.",
                line,
            )
        })?;

        if argc != expected_arity {
            return Err(runtime_error(
                RUNTIME_WRONG_CALL_ARITY,
                format!("Function `{name}` expects {expected_arity} arguments, got {argc}"),
                "Call the function with the declared number of arguments.",
                line,
            ));
        }

        if self.stack.len() < argc as usize {
            return Err(internal_error(
                format!(
                    "Call to `{name}` expected {argc} argument(s) on the stack, found {}",
                    self.stack.len()
                ),
                "This indicates invalid bytecode or a VM stack-layout bug. Please report it.",
                line,
            ));
        }

        Ok((code_start, self.stack.len() - argc as usize))
    }

    fn enter_function(
        &mut self,
        code_start: usize,
        base_slot: usize,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.push_call_frame(
            CallFrame {
                return_ip: self.ip,
                base_slot,
            },
            line,
        )?;
        self.ip = code_start;
        Ok(())
    }
}
