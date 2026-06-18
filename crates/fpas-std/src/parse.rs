//! `Std.Parse.*` intrinsic implementations.
//!
//! **Documentation:** `docs/pascal/std/text/parse.md` (from the repository root).

use crate::error::StdError;
use crate::intrinsic_args::{pop_string, pop_value};
use crate::numeric_text::{parse_bool_text, parse_pascal_integer, parse_pascal_real};
use fpas_bytecode::{Intrinsic, ParseIntrinsic, SourceLocation, Value};

pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Parse(ParseIntrinsic::TryInt) => {
            let text = pop_string(pop_value(stack, location)?, location)?;
            stack.push(parse_int_result(&text));
        }
        Intrinsic::Parse(ParseIntrinsic::TryReal) => {
            let text = pop_string(pop_value(stack, location)?, location)?;
            stack.push(parse_real_result(&text));
        }
        Intrinsic::Parse(ParseIntrinsic::TryBool) => {
            let text = pop_string(pop_value(stack, location)?, location)?;
            stack.push(parse_bool_result(&text));
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
    Value::ResultOk(Box::new(value))
}

fn err(message: String) -> Value {
    Value::ResultError(Box::new(Value::Str(message)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn try_int_returns_ok_for_pascal_integer_text() {
        let mut stack = vec![Value::Str(" +1_024 ".to_string())];

        run(Intrinsic::Parse(ParseIntrinsic::TryInt), &mut stack, loc()).unwrap();

        assert_eq!(stack, vec![Value::ResultOk(Box::new(Value::Integer(1024)))]);
    }

    #[test]
    fn try_real_returns_error_for_non_pascal_real_text() {
        let mut stack = vec![Value::Str("1e3".to_string())];

        run(Intrinsic::Parse(ParseIntrinsic::TryReal), &mut stack, loc()).unwrap();

        assert!(matches!(stack.as_slice(), [Value::ResultError(_)]));
    }

    #[test]
    fn try_bool_accepts_case_insensitive_true() {
        let mut stack = vec![Value::Str(" TRUE ".to_string())];

        run(Intrinsic::Parse(ParseIntrinsic::TryBool), &mut stack, loc()).unwrap();

        assert_eq!(stack, vec![Value::ResultOk(Box::new(Value::Boolean(true)))]);
    }
}
