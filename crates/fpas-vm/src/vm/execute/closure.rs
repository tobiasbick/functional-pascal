//! Closure construction and mutable capture cells.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;
use std::sync::{Arc, Mutex};

impl Worker {
    /// Handle `MakeClosure`, `MakeCell`, `CellGet`, and `CellSet`.
    pub(super) fn try_exec_closure_ops(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::MakeClosure(name_idx, capture_count) => {
                let name = self.const_str(name_idx, line)?;
                let count = capture_count as usize;
                if self.stack.len() < count {
                    return Err(crate::vm::internal_error(
                        format!(
                            "MakeClosure expected {count} capture(s) on the stack, found {}",
                            self.stack.len()
                        ),
                        "This indicates invalid bytecode or a compiler capture-layout bug. Please report it.",
                        line,
                    ));
                }
                let start = self.stack.len() - count;
                let captures: Vec<Value> = self.stack.drain(start..).collect();
                // Direct cell captures and nested task-bound function values both
                // make this closure task-bound (mutable state must not cross `go`).
                let task_bound = captures.iter().any(|capture| {
                    matches!(
                        capture,
                        Value::Cell(_)
                            | Value::Function {
                                task_bound: true,
                                ..
                            }
                    )
                });
                self.push(Value::Function {
                    name,
                    captures,
                    task_bound,
                })?;
                Ok(true)
            }
            Op::MakeCell => {
                let value = self.pop(line)?;
                self.push(Value::Cell(Arc::new(Mutex::new(value))))?;
                Ok(true)
            }
            Op::CellGet => {
                let cell = self.pop(line)?;
                match cell {
                    Value::Cell(cell) => {
                        let value = cell
                            .lock()
                            .map_err(|_| {
                                crate::vm::internal_error(
                                    "CellGet found a poisoned capture cell",
                                    "This indicates a concurrent panic while mutating a capture. Please report it.",
                                    line,
                                )
                            })?
                            .clone();
                        self.push(value)?;
                        Ok(true)
                    }
                    other => Err(runtime_error(
                        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                        format!("CellGet expected a cell, got `{}`", other.type_name()),
                        "Only mutable capture cells support CellGet.",
                        line,
                    )),
                }
            }
            Op::CellSet => {
                let value = self.pop(line)?;
                let cell = self.pop(line)?;
                match cell {
                    Value::Cell(cell) => {
                        let mut guard = cell.lock().map_err(|_| {
                            crate::vm::internal_error(
                                "CellSet found a poisoned capture cell",
                                "This indicates a concurrent panic while mutating a capture. Please report it.",
                                line,
                            )
                        })?;
                        *guard = value;
                        self.push(Value::Unit)?;
                        Ok(true)
                    }
                    other => Err(runtime_error(
                        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                        format!("CellSet expected a cell, got `{}`", other.type_name()),
                        "Only mutable capture cells support CellSet.",
                        line,
                    )),
                }
            }
            _ => Ok(false),
        }
    }
}
