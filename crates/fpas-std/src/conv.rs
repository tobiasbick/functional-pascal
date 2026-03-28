//! `Std.Conv.*` intrinsic implementations.
//!
//! **Documentation:** `docs/pascal/std/conv.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-bytecode::Intrinsic`, `fpas-compiler`, and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_runtime_error};
use crate::helpers::{pop_char, pop_int, pop_real, pop_string, pop_value};
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_CONVERSION_FAILURE;

pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::ConvIntToStr => {
            let n = pop_int(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(format!("{n}")));
        }
        Intrinsic::ConvStrToInt => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            let n = s.trim().parse::<i64>().map_err(|_| {
                std_runtime_error(
                    RUNTIME_CONVERSION_FAILURE,
                    format!("StrToInt: invalid integer `{s}`"),
                    "Provide a valid base-10 integer string, for example `42` or `-7`.",
                    location,
                )
            })?;
            stack.push(Value::Integer(n));
        }
        Intrinsic::ConvRealToStr => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(format!("{r}")));
        }
        Intrinsic::ConvStrToReal => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            let r = s.trim().parse::<f64>().map_err(|_| {
                std_runtime_error(
                    RUNTIME_CONVERSION_FAILURE,
                    format!("StrToReal: invalid real `{s}`"),
                    "Provide a valid real literal string, for example `3.14` or `-2.0`.",
                    location,
                )
            })?;
            stack.push(Value::Real(r));
        }
        Intrinsic::ConvCharToStr => {
            let c = pop_char(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(c.to_string()));
        }
        Intrinsic::ConvIntToReal => {
            let n = pop_int(pop_value(stack, location)?, location)?;
            stack.push(Value::Real(n as f64));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}
