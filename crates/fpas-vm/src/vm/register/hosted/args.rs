//! Borrowed `Std.Args` register intrinsics.

use fpas_bytecode::{ArgsIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::VmError;
use super::super::worker::RegisterWorker;

impl RegisterWorker {
    pub(super) fn execute_args_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        match intrinsic {
            Intrinsic::Args(ArgsIntrinsic::ParamCount) => {
                require_count(arguments, 0, location)?;
                let count = i64::try_from(self.hosted.program_args.len()).map_err(|_| {
                    self.runtime_error(
                        RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                        "Program argument count exceeds the FPAS integer range",
                        "Pass fewer process arguments to this program.",
                    )
                })?;
                Ok(Some(Some(Value::Integer(count))))
            }
            Intrinsic::Args(ArgsIntrinsic::ParamStr) => {
                require_count(arguments, 1, location)?;
                let index = match &arguments[0] {
                    Value::Integer(value) if *value >= 0 => usize::try_from(*value).ok(),
                    Value::Integer(_) => None,
                    actual => {
                        return Err(self.runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            format!(
                                "Std.Args.ParamStr expected integer, got {}",
                                actual.type_name()
                            ),
                            "Pass a non-negative integer argument index.",
                        ));
                    }
                }
                .ok_or_else(|| {
                    self.runtime_error(
                        RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                        "Program argument index is outside the host index range",
                        "Use a non-negative index smaller than Std.Args.ParamCount().",
                    )
                })?;
                let value = self.hosted.program_args.get(index).ok_or_else(|| {
                    self.runtime_error(
                        RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                        format!(
                            "Program argument index {index} out of bounds (count {})",
                            self.hosted.program_args.len()
                        ),
                        "Call Std.Args.ParamCount() before indexing program arguments.",
                    )
                })?;
                Ok(Some(Some(Value::Str(value.as_str().into()))))
            }
            _ => Ok(None),
        }
    }
}

fn require_count(
    arguments: &[Value],
    expected: usize,
    location: SourceLocation,
) -> Result<(), VmError> {
    if arguments.len() == expected {
        return Ok(());
    }
    Err(fpas_diagnostics::Diagnostic::error(
        fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!(
            "Hosted intrinsic expected {expected} arguments, got {}",
            arguments.len()
        ),
        Some("Check the compiler intrinsic signature and register argument count.".to_string()),
        fpas_diagnostics::SourceSpan::new_with_source(
            0,
            1,
            location.line(),
            location.column(),
            location.source_id(),
        ),
    ))
}
