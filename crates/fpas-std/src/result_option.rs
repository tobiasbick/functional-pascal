//! `Std.Results.*` and `Std.Options.*` intrinsic implementations.
//!
//! **Documentation:** `docs/pascal/language/types/result-operations.md` and `docs/pascal/language/types/option-operations.md` (from the repository root).

use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::{IntrinsicCall, pop_value};
use fpas_bytecode::{Intrinsic, OptionIntrinsic, ResultIntrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_UNWRAP_FAILURE;

/// Executes explicit unwrapping with runtime failure on Error or None.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Result(ResultIntrinsic::Unwrap) => {
            let val = pop_value(call, location)?;
            match val {
                Value::ResultOk(inner) => call.push((**inner).clone()),
                Value::ResultError(e) => {
                    return Err(std_runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        format!("Called Result.Unwrap on Error({e})"),
                        "Handle expected errors with `case Value of when Ok(const Item): ...; when Error(const Message): ...; end case;`. Use Unwrap when runtime failure is intended.",
                        location,
                    ));
                }
                _ => {
                    return Err(std_runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        format!(
                            "Result.Unwrap expects a Result value, got {}",
                            val.type_name()
                        ),
                        "Call Value.Unwrap() on a Result value (Ok or Error).",
                        location,
                    ));
                }
            }
        }
        Intrinsic::Option(OptionIntrinsic::Unwrap) => {
            let val = pop_value(call, location)?;
            match val {
                Value::OptionSome(inner) => call.push((**inner).clone()),
                Value::OptionNone => {
                    return Err(std_runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        "Called Option.Unwrap on None",
                        "Handle expected absence with `case Value of when Some(const Item): ...; when None: ...; end case;`. Use Unwrap when runtime failure is intended.",
                        location,
                    ));
                }
                _ => {
                    return Err(std_runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        format!(
                            "Option.Unwrap expects an Option value, got {}",
                            val.type_name()
                        ),
                        "Call Value.Unwrap() on an Option value (Some or None).",
                        location,
                    ));
                }
            }
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}
