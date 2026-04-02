//! `Std.Conv.*` intrinsic implementations.
//!
//! **Documentation:** `docs/pascal/std/conv.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-bytecode::Intrinsic`, `fpas-compiler`, and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_runtime_error};
use crate::helpers::{pop_bool, pop_char, pop_int, pop_real, pop_string, pop_value};
use crate::numeric_text::parse_pascal_real;
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
            let r = parse_pascal_real(&s).ok_or_else(|| {
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
        Intrinsic::ConvBoolToStr => {
            let b = pop_bool(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(if b {
                "true".to_string()
            } else {
                "false".to_string()
            }));
        }
        Intrinsic::ConvStrToBool => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            match s.trim().to_lowercase().as_str() {
                "true" => stack.push(Value::Boolean(true)),
                "false" => stack.push(Value::Boolean(false)),
                _ => {
                    return Err(std_runtime_error(
                        RUNTIME_CONVERSION_FAILURE,
                        format!("StrToBool: invalid boolean `{s}`"),
                        "Provide `true` or `false` (case-insensitive).",
                        location,
                    ));
                }
            }
        }
        Intrinsic::ConvIntToHex => {
            let digits = pop_int(pop_value(stack, location)?, location)?;
            let n = pop_int(pop_value(stack, location)?, location)?;
            if digits < 0 {
                return Err(std_runtime_error(
                    RUNTIME_CONVERSION_FAILURE,
                    format!("IntToHex: Digits must be >= 0, got {digits}"),
                    "Pass a non-negative Digits value to Std.Conv.IntToHex.",
                    location,
                ));
            }

            let width = digits as usize;
            let formatted = if n < 0 {
                let magnitude = n.unsigned_abs();
                format!("-{:0width$X}", magnitude, width = width)
            } else {
                format!("{:0width$X}", n, width = width)
            };
            stack.push(Value::Str(formatted));
        }
        Intrinsic::ConvHexToInt => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            let trimmed = s.trim();
            let trimmed = trimmed
                .strip_prefix("0x")
                .or_else(|| trimmed.strip_prefix("0X"))
                .or_else(|| trimmed.strip_prefix('$'))
                .unwrap_or(trimmed);
            let n = i64::from_str_radix(trimmed, 16).map_err(|_| {
                std_runtime_error(
                    RUNTIME_CONVERSION_FAILURE,
                    format!("HexToInt: invalid hex `{s}`"),
                    "Provide a valid hexadecimal string, for example `FF` or `0x1A`.",
                    location,
                )
            })?;
            stack.push(Value::Integer(n));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}
