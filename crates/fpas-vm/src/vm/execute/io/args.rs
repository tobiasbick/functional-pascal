use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, internal_error, runtime_error};
use fpas_bytecode::{ArgsIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS;

impl Worker {
    pub(super) fn try_exec_args_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Args(ArgsIntrinsic::ParamCount) => {
                let count = i64::try_from(self.shared.program_args.len()).map_err(|_| {
                    internal_error(
                        "Program argument count exceeds FPAS integer range",
                        "This indicates a host/runtime argument handling bug. Please report it.",
                        line,
                    )
                })?;
                self.push(Value::Integer(count))?;
                Ok(true)
            }
            Intrinsic::Args(ArgsIntrinsic::ParamStr) => {
                let raw_index = self.pop_int(line)?;
                let index = program_arg_index(raw_index, line)?;
                let Some(value) = self.shared.program_args.get(index).cloned() else {
                    return Err(runtime_error(
                        RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                        format!(
                            "Program argument index {index} out of bounds (count {})",
                            self.shared.program_args.len()
                        ),
                        "Call Std.Args.ParamCount() before indexing program arguments.",
                        line,
                    ));
                };
                self.push(Value::Str(value.into()))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn program_arg_index(raw_index: i64, line: SourceLocation) -> Result<usize, VmError> {
    if raw_index < 0 {
        return Err(runtime_error(
            RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
            format!("Negative program argument index {raw_index}"),
            "Program argument indices are non-negative integers (0-based).",
            line,
        ));
    }
    usize::try_from(raw_index).map_err(|_| {
        runtime_error(
            RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
            format!("Program argument index {raw_index} is out of range"),
            "Use an index that fits the host pointer size.",
            line,
        )
    })
}
