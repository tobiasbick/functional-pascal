//! `Std.Env` runtime implementation.
//!
//! **Documentation:** `docs/pascal/std/host/env.md` (from the repository root).

use crate::error::StdError;
use crate::intrinsic_args::{IntrinsicCall, pop_string, pop_value};
use fpas_bytecode::{EnvIntrinsic, Intrinsic, SourceLocation, Value};

/// Execute a `Std.Env` intrinsic and return `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Env(EnvIntrinsic::Get) => {
            let name = pop_string(pop_value(call, location)?, location)?;
            match std::env::var_os(name) {
                Some(value) => call.push(Value::OptionSome(Box::new(Value::Str(
                    value.to_string_lossy().into_owned().into(),
                )))),
                None => call.push(Value::OptionNone),
            }
        }
        Intrinsic::Env(EnvIntrinsic::Exists) => {
            let name = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Boolean(std::env::var_os(name).is_some()));
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
    fn get_returns_none_when_variable_is_missing() {
        let mut stack = vec![Value::Str("__FPAS_ENV_TEST_MISSING_903B5CF4__".into())];

        crate::run_intrinsic(
            Intrinsic::Env(EnvIntrinsic::Get),
            &mut stack,
            test_location(),
        )
        .unwrap();

        assert_eq!(stack, vec![Value::OptionNone]);
    }

    #[test]
    fn exists_returns_false_when_variable_is_missing() {
        let mut stack = vec![Value::Str("__FPAS_ENV_TEST_MISSING_48872D88__".into())];

        crate::run_intrinsic(
            Intrinsic::Env(EnvIntrinsic::Exists),
            &mut stack,
            test_location(),
        )
        .unwrap();

        assert_eq!(stack, vec![Value::Boolean(false)]);
    }
}
