//! `Std.Proc` runtime implementation.
//!
//! Blocking host process execution safe to call from `go` tasks.
//!
//! **Documentation:** `docs/pascal/std/proc.md` (from the repository root).

use crate::error::StdError;
use crate::helpers::{pop_array, pop_string, pop_value};
use fpas_bytecode::{Intrinsic, ProcIntrinsic, SourceLocation, Value};
use std::process::Command;

/// Execute a `Std.Proc` intrinsic and return `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Proc(ProcIntrinsic::Run) => {
            let args = pop_string_array(pop_value(stack, location)?, location)?;
            let command = pop_string(pop_value(stack, location)?, location)?;
            stack.push(run_process(&command, &args));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn pop_string_array(value: Value, location: SourceLocation) -> Result<Vec<String>, StdError> {
    pop_array(value, location)?
        .into_iter()
        .map(|value| pop_string(value, location))
        .collect()
}

fn run_process(command: &str, args: &[String]) -> Value {
    match Command::new(command).args(args).status() {
        Ok(status) => match status.code() {
            Some(code) => Value::ResultOk(Box::new(Value::Integer(i64::from(code)))),
            None => Value::ResultError(Box::new(Value::Str(
                "process terminated without an exit code".into(),
            ))),
        },
        Err(error) => Value::ResultError(Box::new(Value::Str(error.to_string()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn run_proc(stack: &mut Vec<Value>) {
        run(Intrinsic::Proc(ProcIntrinsic::Run), stack, test_location()).unwrap();
    }

    #[test]
    fn run_returns_exit_code_for_successful_process() {
        let (command, args) = successful_process_fixture();
        let mut stack = vec![Value::Str(command), Value::Array(args)];

        run_proc(&mut stack);

        assert_eq!(stack, vec![Value::ResultOk(Box::new(Value::Integer(0)))]);
    }

    #[cfg(windows)]
    fn successful_process_fixture() -> (String, Vec<Value>) {
        (
            "cmd".into(),
            vec![Value::Str("/C".into()), Value::Str("exit 0".into())],
        )
    }

    #[cfg(not(windows))]
    fn successful_process_fixture() -> (String, Vec<Value>) {
        (
            "sh".into(),
            vec![Value::Str("-c".into()), Value::Str("exit 0".into())],
        )
    }

    #[test]
    fn run_returns_error_for_missing_command() {
        let mut stack = vec![
            Value::Str("__fpas_proc_missing_command_8f21d2f4__".into()),
            Value::Array(Vec::new()),
        ];

        run_proc(&mut stack);

        assert!(matches!(stack[0], Value::ResultError(_)));
    }
}
