//! `Std.Result.*` and `Std.Option.*` intrinsic implementations.
//!
//! **Documentation:** `docs/pascal/std/result.md` and `docs/pascal/std/option.md` (from the repository root).

use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::pop_value;
use fpas_bytecode::{Intrinsic, OptionIntrinsic, ResultIntrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_UNWRAP_FAILURE;

pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Result(ResultIntrinsic::Unwrap) => {
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
        Intrinsic::Result(ResultIntrinsic::UnwrapOr) => {
            let default = pop_value(stack, location)?;
            let val = pop_value(stack, location)?;
            match val {
                Value::ResultOk(inner) => stack.push(*inner),
                Value::ResultError(_) => stack.push(default),
                _ => {
                    return Err(std_runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        format!(
                            "Std.Result.UnwrapOr expects a Result value, got {}",
                            val.type_name()
                        ),
                        "Pass a Result value (Ok or Error) as the first argument to Std.Result.UnwrapOr.",
                        location,
                    ));
                }
            }
        }
        Intrinsic::Result(ResultIntrinsic::IsOk) => {
            let val = pop_value(stack, location)?;
            stack.push(Value::Boolean(matches!(val, Value::ResultOk(_))));
        }
        Intrinsic::Result(ResultIntrinsic::IsError) => {
            let val = pop_value(stack, location)?;
            stack.push(Value::Boolean(matches!(val, Value::ResultError(_))));
        }
        Intrinsic::Option(OptionIntrinsic::Unwrap) => {
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
        Intrinsic::Option(OptionIntrinsic::UnwrapOr) => {
            let default = pop_value(stack, location)?;
            let val = pop_value(stack, location)?;
            match val {
                Value::OptionSome(inner) => stack.push(*inner),
                Value::OptionNone => stack.push(default),
                _ => {
                    return Err(std_runtime_error(
                        RUNTIME_UNWRAP_FAILURE,
                        format!(
                            "Std.Option.UnwrapOr expects an Option value, got {}",
                            val.type_name()
                        ),
                        "Pass an Option value (Some or None) as the first argument to Std.Option.UnwrapOr.",
                        location,
                    ));
                }
            }
        }
        Intrinsic::Option(OptionIntrinsic::IsSome) => {
            let val = pop_value(stack, location)?;
            stack.push(Value::Boolean(matches!(val, Value::OptionSome(_))));
        }
        Intrinsic::Option(OptionIntrinsic::IsNone) => {
            let val = pop_value(stack, location)?;
            stack.push(Value::Boolean(matches!(val, Value::OptionNone)));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn result_unwrap_or_rejects_non_result_values() {
        let mut stack = vec![Value::Integer(1), Value::Integer(99)];

        let error = run(
            Intrinsic::Result(ResultIntrinsic::UnwrapOr),
            &mut stack,
            test_location(),
        )
        .unwrap_err();

        assert!(
            error.message.contains("expects a Result value"),
            "{}",
            error.message
        );
    }

    #[test]
    fn option_unwrap_or_rejects_non_option_values() {
        let mut stack = vec![Value::Integer(1), Value::Integer(99)];

        let error = run(
            Intrinsic::Option(OptionIntrinsic::UnwrapOr),
            &mut stack,
            test_location(),
        )
        .unwrap_err();

        assert!(
            error.message.contains("expects an Option value"),
            "{}",
            error.message
        );
    }
}
