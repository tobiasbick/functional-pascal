//! `Std.Parse.*` intrinsic implementations.
//!
//! **Documentation:** `docs/pascal/std/text/parse.md` (from the repository root).

use crate::stdlib::boolean_text::parse_bool_text;
use crate::stdlib::error::StdError;
use crate::stdlib::intrinsic_args::{IntrinsicCall, expect_str, pop_value};
use crate::{Intrinsic, ParseIntrinsic, SourceLocation, Value};
use fpas_numeric::text::{integer as parse_pascal_integer, real as parse_pascal_real};

/// Executes parsers using the single shared numeric grammar.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    use crate::stdlib::native_errors::{NativeErrorFamily, value};
    use fpas_numeric::text::TextError;
    let Intrinsic::Parse(operation) = intrinsic else {
        return Ok(None);
    };
    let text = expect_str(pop_value(call, location)?, location)?;
    let (target, result) = match operation {
        ParseIntrinsic::Int => ("integer", parse_pascal_integer(text).map(Value::Integer)),
        ParseIntrinsic::Real => ("real", parse_pascal_real(text).map(Value::Real)),
        ParseIntrinsic::HexInt => (
            "hexadecimal integer",
            fpas_numeric::text::hex_integer(text).map(Value::Integer),
        ),
        ParseIntrinsic::Bool => (
            "boolean",
            parse_bool_text(text)
                .map(Value::Boolean)
                .ok_or(TextError::InvalidSyntax),
        ),
    };
    let result = match result {
        Ok(value) => Value::result_ok(value),
        Err(reason) => Value::result_error(value(
            call.factory(),
            NativeErrorFamily::Parse,
            match reason {
                TextError::InvalidSyntax => 0,
                TextError::OutOfRange => 1,
            },
            vec![
                Value::Str(target.into()),
                Value::Str(match operation {
                    ParseIntrinsic::Int => format!("invalid integer `{text}`; expected Pascal integer text such as `42`, `-7`, or `1_000`"),
                    ParseIntrinsic::Real => format!("invalid real `{text}`; expected Pascal real text such as `3.14`, `-2.0`, or `1.0e3`"),
                    ParseIntrinsic::Bool => format!("invalid boolean `{text}`; expected `true` or `false`"),
                    ParseIntrinsic::HexInt => format!("invalid hexadecimal integer `{text}`; expected hex text such as `FF`, `$FF`, or `0x1A`"),
                }.into()),
            ],
            location,
        )?),
    };
    call.push(result);
    Ok(Some(()))
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

        crate::stdlib::execute_test_intrinsic(
            Intrinsic::Parse(ParseIntrinsic::Int),
            &mut stack,
            loc(),
        )
        .unwrap();

        assert_eq!(stack, vec![Value::result_ok(Value::Integer(1024))]);
    }

    #[test]
    fn try_real_returns_error_for_non_pascal_real_text() {
        let mut stack = vec![Value::Str("1e3".into())];

        crate::stdlib::execute_test_intrinsic(
            Intrinsic::Parse(ParseIntrinsic::Real),
            &mut stack,
            loc(),
        )
        .unwrap();

        assert!(matches!(stack.as_slice(), [Value::ResultError(_)]));
    }

    #[test]
    fn try_bool_accepts_case_insensitive_true() {
        let mut stack = vec![Value::Str(" TRUE ".into())];

        crate::stdlib::execute_test_intrinsic(
            Intrinsic::Parse(ParseIntrinsic::Bool),
            &mut stack,
            loc(),
        )
        .unwrap();

        assert_eq!(stack, vec![Value::result_ok(Value::Boolean(true))]);
    }
}

#[cfg(test)]
mod allocations;
