//! `Std.Conv.*` intrinsic implementations.
//!
//! **Documentation:** `docs/pascal/std/text/conv.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-bytecode::Intrinsic`, `fpas-compiler`, and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::{pop_bool, pop_int, pop_real, pop_string, pop_value};
use crate::numeric_text::{parse_bool_text, parse_pascal_integer, parse_pascal_real};
use fpas_bytecode::{ConvIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_CONVERSION_FAILURE;

pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Conv(ConvIntrinsic::IntToStr) => {
            let n = pop_int(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(format!("{n}")));
        }
        Intrinsic::Conv(ConvIntrinsic::StrToInt) => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            let n = parse_pascal_integer(&s).ok_or_else(|| {
                std_runtime_error(
                    RUNTIME_CONVERSION_FAILURE,
                    format!("StrToInt: invalid integer `{s}`"),
                    "Provide a valid Pascal integer string, for example `42`, `-7`, or `1_000`.",
                    location,
                )
            })?;
            stack.push(Value::Integer(n));
        }
        Intrinsic::Conv(ConvIntrinsic::RealToStr) => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(format!("{r}")));
        }
        Intrinsic::Conv(ConvIntrinsic::StrToReal) => {
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
        Intrinsic::Conv(ConvIntrinsic::IntToReal) => {
            let n = pop_int(pop_value(stack, location)?, location)?;
            stack.push(Value::Real(n as f64));
        }
        Intrinsic::Conv(ConvIntrinsic::BoolToStr) => {
            let b = pop_bool(pop_value(stack, location)?, location)?;
            stack.push(Value::Str(if b {
                "true".to_string()
            } else {
                "false".to_string()
            }));
        }
        Intrinsic::Conv(ConvIntrinsic::StrToBool) => {
            let s = pop_string(pop_value(stack, location)?, location)?;
            let b = parse_bool_text(&s).ok_or_else(|| {
                std_runtime_error(
                    RUNTIME_CONVERSION_FAILURE,
                    format!("StrToBool: invalid boolean `{s}`"),
                    "Provide `true` or `false` (case-insensitive).",
                    location,
                )
            })?;
            stack.push(Value::Boolean(b));
        }
        Intrinsic::Conv(ConvIntrinsic::IntToHex) => {
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
        Intrinsic::Conv(ConvIntrinsic::HexToInt) => {
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
