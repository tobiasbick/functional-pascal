//! `Std.Parse.*` intrinsic implementations.
//!
//! **Documentation:** `docs/pascal/std/text/parse.md` (from the repository root).

use crate::boolean_text::parse_bool_text;
use crate::error::StdError;
use crate::intrinsic_args::{IntrinsicCall, expect_str, pop_value};
use fpas_bytecode::{Intrinsic, ParseIntrinsic, SourceLocation, Value};
use fpas_numeric::text::{integer as parse_pascal_integer, real as parse_pascal_real};

/// Executes Result-returning parsing APIs with the shared numeric grammar and bounds.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Parse(ParseIntrinsic::Int) => {
            let text = expect_str(pop_value(call, location)?, location)?;
            call.push(parse_int_result(text));
        }
        Intrinsic::Parse(ParseIntrinsic::Real) => {
            let text = expect_str(pop_value(call, location)?, location)?;
            call.push(parse_real_result(text));
        }
        Intrinsic::Parse(ParseIntrinsic::Bool) => {
            let text = expect_str(pop_value(call, location)?, location)?;
            call.push(parse_bool_result(text));
        }
        Intrinsic::Parse(ParseIntrinsic::HexInt) => {
            let text = expect_str(pop_value(call, location)?, location)?;
            call.push(match fpas_numeric::text::hex_integer(text) {
                Some(value) => ok(Value::Integer(value)),
                None => err(format!(
                    "invalid hexadecimal integer `{text}`; expected hex text such as `FF`, `$FF`, or `0x1A`"
                )),
            });
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn parse_int_result(text: &str) -> Value {
    match parse_pascal_integer(text) {
        Some(value) => ok(Value::Integer(value)),
        None => err(format!(
            "invalid integer `{text}`; expected Pascal integer text such as `42`, `-7`, or `1_000`"
        )),
    }
}

fn parse_real_result(text: &str) -> Value {
    match parse_pascal_real(text) {
        Some(value) => ok(Value::Real(value)),
        None => err(format!(
            "invalid real `{text}`; expected Pascal real text such as `3.14`, `-2.0`, or `1.0e3`"
        )),
    }
}

fn parse_bool_result(text: &str) -> Value {
    match parse_bool_text(text) {
        Some(value) => ok(Value::Boolean(value)),
        None => err(format!(
            "invalid boolean `{text}`; expected `true` or `false`"
        )),
    }
}

fn ok(value: Value) -> Value {
    Value::result_ok(value)
}

fn err(message: String) -> Value {
    Value::result_error(Value::Str(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn try_int_returns_ok_for_pascal_integer_text() {
        let mut stack = vec![Value::Str(" +1_024 ".into())];

        crate::execute_test_intrinsic(Intrinsic::Parse(ParseIntrinsic::Int), &mut stack, loc())
            .unwrap();

        assert_eq!(stack, vec![Value::result_ok(Value::Integer(1024))]);
    }

    #[test]
    fn try_real_returns_error_for_non_pascal_real_text() {
        let mut stack = vec![Value::Str("1e3".into())];

        crate::execute_test_intrinsic(Intrinsic::Parse(ParseIntrinsic::Real), &mut stack, loc())
            .unwrap();

        assert!(matches!(stack.as_slice(), [Value::ResultError(_)]));
    }

    #[test]
    fn try_bool_accepts_case_insensitive_true() {
        let mut stack = vec![Value::Str(" TRUE ".into())];

        crate::execute_test_intrinsic(Intrinsic::Parse(ParseIntrinsic::Bool), &mut stack, loc())
            .unwrap();

        assert_eq!(stack, vec![Value::result_ok(Value::Boolean(true))]);
    }
}

#[cfg(test)]
mod allocations;
