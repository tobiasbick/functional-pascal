//! VM callback implementations for `Std.Result.{Map,AndThen,OrElse}` and `Std.Option.{Map,AndThen,OrElse}`.
//!
//! **Documentation:** `docs/pascal/std/result.md` and `docs/pascal/std/option.md` (from the repository root).

use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

impl Worker {
    /// `Std.Result.Map(R, F)` — `Ok(v)` → `Ok(F(v))`, `Error(e)` passthrough.
    pub(super) fn exec_result_map(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let val = self.pop(line)?;
        match val {
            Value::ResultOk(inner) => {
                let mapped = self.call_function_sync(&func, &[*inner], line)?;
                self.push(Value::ResultOk(Box::new(mapped)))?;
            }
            Value::ResultError(_) => {
                self.push(val)?;
            }
            other => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "Std.Result.Map expects a Result value, got `{}`",
                        other.type_name()
                    ),
                    "Pass a Result value (Ok or Error) as the first argument.",
                    line,
                ));
            }
        }
        Ok(())
    }

    /// `Std.Result.AndThen(R, F)` — `Ok(v)` → `F(v)`, `Error(e)` passthrough.
    pub(super) fn exec_result_and_then(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let val = self.pop(line)?;
        match val {
            Value::ResultOk(inner) => {
                let result = self.call_function_sync(&func, &[*inner], line)?;
                self.push(result)?;
            }
            Value::ResultError(_) => {
                self.push(val)?;
            }
            other => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "Std.Result.AndThen expects a Result value, got `{}`",
                        other.type_name()
                    ),
                    "Pass a Result value (Ok or Error) as the first argument.",
                    line,
                ));
            }
        }
        Ok(())
    }

    /// `Std.Result.OrElse(R, F)` — `Ok(v)` passthrough, `Error(e)` → `F(e)`.
    pub(super) fn exec_result_or_else(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let val = self.pop(line)?;
        match val {
            Value::ResultOk(_) => {
                self.push(val)?;
            }
            Value::ResultError(inner) => {
                let result = self.call_function_sync(&func, &[*inner], line)?;
                self.push(result)?;
            }
            other => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "Std.Result.OrElse expects a Result value, got `{}`",
                        other.type_name()
                    ),
                    "Pass a Result value (Ok or Error) as the first argument.",
                    line,
                ));
            }
        }
        Ok(())
    }

    /// `Std.Option.Map(O, F)` — `Some(v)` → `Some(F(v))`, `None` passthrough.
    pub(super) fn exec_option_map(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let val = self.pop(line)?;
        match val {
            Value::OptionSome(inner) => {
                let mapped = self.call_function_sync(&func, &[*inner], line)?;
                self.push(Value::OptionSome(Box::new(mapped)))?;
            }
            Value::OptionNone => {
                self.push(Value::OptionNone)?;
            }
            other => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "Std.Option.Map expects an Option value, got `{}`",
                        other.type_name()
                    ),
                    "Pass an Option value (Some or None) as the first argument.",
                    line,
                ));
            }
        }
        Ok(())
    }

    /// `Std.Option.AndThen(O, F)` — `Some(v)` → `F(v)`, `None` passthrough.
    pub(super) fn exec_option_and_then(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let val = self.pop(line)?;
        match val {
            Value::OptionSome(inner) => {
                let result = self.call_function_sync(&func, &[*inner], line)?;
                self.push(result)?;
            }
            Value::OptionNone => {
                self.push(Value::OptionNone)?;
            }
            other => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "Std.Option.AndThen expects an Option value, got `{}`",
                        other.type_name()
                    ),
                    "Pass an Option value (Some or None) as the first argument.",
                    line,
                ));
            }
        }
        Ok(())
    }

    /// `Std.Option.OrElse(O, F)` — `Some(v)` passthrough, `None` → `F()`.
    pub(super) fn exec_option_or_else(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let val = self.pop(line)?;
        match val {
            Value::OptionSome(_) => {
                self.push(val)?;
            }
            Value::OptionNone => {
                let result = self.call_function_sync(&func, &[], line)?;
                self.push(result)?;
            }
            other => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "Std.Option.OrElse expects an Option value, got `{}`",
                        other.type_name()
                    ),
                    "Pass an Option value (Some or None) as the first argument.",
                    line,
                ));
            }
        }
        Ok(())
    }
}
