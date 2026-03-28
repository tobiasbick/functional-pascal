//! `Std.Result.*` and `Std.Option.*` intrinsic implementations.
//!
//! **Documentation:** `docs/pascal/std/result.md` and `docs/pascal/std/option.md` (from the repository root).

use crate::error::{StdError, std_runtime_error};
use crate::helpers::pop_value;
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_UNWRAP_FAILURE;

pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::ResultUnwrap => {
            let val = pop_value(stack, location)?;
            match val {
                Value::ResultOk(inner) => stack.push(*inner),
                Value::ResultError(e) => {
                    return Err(std_runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        format!("Called Std.Result.Unwrap on Error({e})"),
                        "Check with Std.Result.IsOk before unwrapping, or use Std.Result.UnwrapOr.",
                        location,
                    ));
                }
                _ => {
                    return Err(std_runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        format!(
                            "Std.Result.Unwrap expects a Result value, got {}",
                            val.type_name()
                        ),
                        "Pass a Result value (Ok or Error) to Std.Result.Unwrap.",
                        location,
                    ));
                }
            }
        }
        Intrinsic::ResultUnwrapOr => {
            let default = pop_value(stack, location)?;
            let val = pop_value(stack, location)?;
            match val {
                Value::ResultOk(inner) => stack.push(*inner),
                Value::ResultError(_) => stack.push(default),
                _ => stack.push(default),
            }
        }
        Intrinsic::ResultIsOk => {
            let val = pop_value(stack, location)?;
            stack.push(Value::Boolean(matches!(val, Value::ResultOk(_))));
        }
        Intrinsic::ResultIsError => {
            let val = pop_value(stack, location)?;
            stack.push(Value::Boolean(matches!(val, Value::ResultError(_))));
        }
        Intrinsic::OptionUnwrap => {
            let val = pop_value(stack, location)?;
            match val {
                Value::OptionSome(inner) => stack.push(*inner),
                Value::OptionNone => {
                    return Err(std_runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        "Called Std.Option.Unwrap on None",
                        "Check with Std.Option.IsSome before unwrapping, or use Std.Option.UnwrapOr.",
                        location,
                    ));
                }
                _ => {
                    return Err(std_runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        format!(
                            "Std.Option.Unwrap expects an Option value, got {}",
                            val.type_name()
                        ),
                        "Pass an Option value (Some or None) to Std.Option.Unwrap.",
                        location,
                    ));
                }
            }
        }
        Intrinsic::OptionUnwrapOr => {
            let default = pop_value(stack, location)?;
            let val = pop_value(stack, location)?;
            match val {
                Value::OptionSome(inner) => stack.push(*inner),
                Value::OptionNone => stack.push(default),
                _ => stack.push(default),
            }
        }
        Intrinsic::OptionIsSome => {
            let val = pop_value(stack, location)?;
            stack.push(Value::Boolean(matches!(val, Value::OptionSome(_))));
        }
        Intrinsic::OptionIsNone => {
            let val = pop_value(stack, location)?;
            stack.push(Value::Boolean(matches!(val, Value::OptionNone)));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}
