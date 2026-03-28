use super::super::diagnostics::VmError;
use super::super::{Worker, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_UNWRAP_FAILURE, RUNTIME_VM_OPERAND_TYPE_MISMATCH};

impl Worker {
    pub(super) fn try_exec_result_option(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::MakeOk => {
                let val = self.pop(line)?;
                self.push(Value::ResultOk(Box::new(val)))?;
                Ok(true)
            }
            Op::MakeErr => {
                let val = self.pop(line)?;
                self.push(Value::ResultError(Box::new(val)))?;
                Ok(true)
            }
            Op::MakeSome => {
                let val = self.pop(line)?;
                self.push(Value::OptionSome(Box::new(val)))?;
                Ok(true)
            }
            Op::MakeNone => {
                self.push(Value::OptionNone)?;
                Ok(true)
            }
            Op::IsResultOk => {
                let val = self.pop(line)?;
                let ok = matches!(val, Value::ResultOk(_) | Value::OptionSome(_));
                self.push(Value::Boolean(ok))?;
                Ok(true)
            }
            Op::IsOptionSome => {
                let val = self.pop(line)?;
                let some = matches!(val, Value::OptionSome(_));
                self.push(Value::Boolean(some))?;
                Ok(true)
            }
            Op::UnwrapOk => {
                let val = self.pop(line)?;
                match val {
                    Value::ResultOk(inner) => {
                        self.push(*inner)?;
                        Ok(true)
                    }
                    Value::OptionSome(inner) => {
                        self.push(*inner)?;
                        Ok(true)
                    }
                    Value::ResultError(_) => Err(runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        "Called unwrap on an Error value",
                        "Check with IsOk before unwrapping, or use unwrap_or.",
                        line,
                    )),
                    Value::OptionNone => Err(runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        "Called unwrap on a None value",
                        "Check with IsSome before unwrapping, or use unwrap_or.",
                        line,
                    )),
                    other => Err(runtime_error(
                        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                        format!(
                            "UnwrapOk expected Result or Option, got {}",
                            other.type_name()
                        ),
                        "Ensure you are unwrapping a Result or Option value.",
                        line,
                    )),
                }
            }
            Op::UnwrapErr => {
                let val = self.pop(line)?;
                match val {
                    Value::ResultError(inner) => {
                        self.push(*inner)?;
                        Ok(true)
                    }
                    Value::ResultOk(_) => Err(runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        "Called unwrap_err on an Ok value",
                        "Check with IsError before unwrapping the error.",
                        line,
                    )),
                    other => Err(runtime_error(
                        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                        format!("UnwrapErr expected Result, got {}", other.type_name()),
                        "Ensure you are unwrapping a Result value.",
                        line,
                    )),
                }
            }
            Op::UnwrapSome => {
                let val = self.pop(line)?;
                match val {
                    Value::OptionSome(inner) => {
                        self.push(*inner)?;
                        Ok(true)
                    }
                    Value::OptionNone => Err(runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        "Called unwrap on a None value",
                        "Check with IsSome before unwrapping, or use unwrap_or.",
                        line,
                    )),
                    other => Err(runtime_error(
                        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                        format!("UnwrapSome expected Option, got {}", other.type_name()),
                        "Ensure you are unwrapping an Option value.",
                        line,
                    )),
                }
            }
            _ => Ok(false),
        }
    }
}
