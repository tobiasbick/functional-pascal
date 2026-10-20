//! `Std.Conv.*` intrinsic implementations.
//!
//! **Documentation:** `docs/pascal/std/text/conv.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-bytecode::Intrinsic`, `fpas-compiler`, and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::{IntrinsicCall, pop_bool, pop_int, pop_real, pop_value};
use crate::limits::checked_collection_len;
use fpas_bytecode::{ConvIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_CONVERSION_FAILURE;

/// Formats typed values and performs explicit numeric conversion.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Conv(ConvIntrinsic::ErrorMatchesVariant) => {
            let expected = crate::intrinsic_args::pop_string(pop_value(call, location)?, location)?;
            let value = pop_value(call, location)?;
            let matches = matches!(&value, Value::Enum(value) if value.body().layout.error.is_some()
                && format!("{}.{}", value.body().layout.type_name, value.body().layout.variant).eq_ignore_ascii_case(&expected));
            call.push(Value::Boolean(matches));
        }
        Intrinsic::Conv(ConvIntrinsic::ErrorMessage) => {
            let value = pop_value(call, location)?;
            if !matches!(&value, Value::Enum(value) if value.body().layout.error.is_some()) {
                return Err(std_runtime_error(
                    RUNTIME_CONVERSION_FAILURE,
                    "Message() requires a native error value",
                    "Use a concrete error family or an Error-constrained receiver.",
                    location,
                ));
            }
            call.push(Value::Str(value.to_string().into()));
        }
        Intrinsic::Conv(ConvIntrinsic::IntToStr) => {
            let n = pop_int(pop_value(call, location)?, location)?;
            call.push(Value::Str(format!("{n}").into()));
        }
        Intrinsic::Conv(ConvIntrinsic::RealToStr) => {
            let r = pop_real(pop_value(call, location)?, location)?;
            let text = fpas_numeric::text::format_real(r).map_err(|error| {
                std_runtime_error(
                    fpas_diagnostics::codes::RUNTIME_NUMERIC_DOMAIN_ERROR,
                    error.message(),
                    error.hint(),
                    location,
                )
            })?;
            call.push(Value::Str(text.into()));
        }
        Intrinsic::Conv(ConvIntrinsic::IntToReal) => {
            let n = pop_int(pop_value(call, location)?, location)?;
            call.push(Value::Real(fpas_numeric::real::from_integer(n)));
        }
        Intrinsic::Conv(ConvIntrinsic::BoolToStr) => {
            let b = pop_bool(pop_value(call, location)?, location)?;
            call.push(Value::Str(
                if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
                .into(),
            ));
        }
        Intrinsic::Conv(ConvIntrinsic::IntToHex) => {
            let digits = pop_int(pop_value(call, location)?, location)?;
            let n = pop_int(pop_value(call, location)?, location)?;
            if digits < 0 {
                return Err(std_runtime_error(
                    RUNTIME_CONVERSION_FAILURE,
                    format!("IntToHex: Digits must be >= 0, got {digits}"),
                    "Pass a non-negative Digits value to Std.Conv.IntToHex.",
                    location,
                ));
            }

            let width = checked_collection_len(digits, location, "Std.Conv.IntToHex")?;
            let formatted = if n < 0 {
                let magnitude = n.unsigned_abs();
                format!("-{:0width$X}", magnitude, width = width)
            } else {
                format!("{:0width$X}", n, width = width)
            };
            call.push(Value::Str(formatted.into()));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::MAX_COLLECTION_LEN;
    use fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn run_conv(intrinsic: ConvIntrinsic, stack: &mut Vec<Value>) -> Result<(), StdError> {
        crate::execute_test_intrinsic(Intrinsic::Conv(intrinsic), stack, loc()).map(|_| ())
    }

    #[test]
    fn int_to_str_formats_integer() {
        let mut stack = vec![Value::Integer(-42)];
        run_conv(ConvIntrinsic::IntToStr, &mut stack).unwrap();
        assert_eq!(stack, vec![Value::Str("-42".into())]);
    }

    #[test]
    fn int_to_hex_rejects_digits_above_limit() {
        let mut stack = vec![Value::Integer(1), Value::Integer(MAX_COLLECTION_LEN + 1)];
        let err = run_conv(ConvIntrinsic::IntToHex, &mut stack)
            .expect_err("IntToHex must reject a width above the limit");
        assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
    }
}
