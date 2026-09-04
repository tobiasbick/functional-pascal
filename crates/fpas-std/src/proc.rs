//! `Std.Proc` runtime implementation.
//!
//! Blocking host process execution safe to call from `go` tasks.
//!
//! **Documentation:** `docs/pascal/std/host/proc.md` (from the repository root).

use crate::error::StdError;
use crate::intrinsic_args::{IntrinsicCall, pop_array, pop_string, pop_value};
use crate::std_symbols as s;
use fpas_bytecode::{Intrinsic, ProcIntrinsic, SourceLocation, Value};
use std::env;
use std::process::Command;

/// Execute a `Std.Proc` intrinsic and return `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Proc(ProcIntrinsic::CurrentExecutable) => {
            call.push(current_executable());
        }
        Intrinsic::Proc(ProcIntrinsic::Run) => {
            let args = pop_string_array(pop_value(call, location)?, location)?;
            let command = pop_string(pop_value(call, location)?, location)?;
            call.push(run_process(&command, &args));
        }
        Intrinsic::Proc(ProcIntrinsic::RunCapture) => {
            let args = pop_string_array(pop_value(call, location)?, location)?;
            let command = pop_string(pop_value(call, location)?, location)?;
            let result = run_process_capture(&command, &args, call, location)?;
            call.push(result);
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn pop_string_array(value: &Value, location: SourceLocation) -> Result<Vec<String>, StdError> {
    pop_array(value, location)?
        .into_iter()
        .map(|value| pop_string(&value, location))
        .collect()
}

fn run_process(command: &str, args: &[String]) -> Value {
    match Command::new(command).args(args).status() {
        Ok(status) => match status.code() {
            Some(code) => Value::result_ok(Value::Integer(i64::from(code))),
            None => {
                Value::result_error(Value::Str("process terminated without an exit code".into()))
            }
        },
        Err(error) => Value::result_error(Value::Str(error.to_string().into())),
    }
}

fn current_executable() -> Value {
    match env::current_exe() {
        Ok(path) => Value::result_ok(Value::Str(path.to_string_lossy().into_owned().into())),
        Err(error) => Value::result_error(Value::Str(error.to_string().into())),
    }
}

fn run_process_capture(
    command: &str,
    args: &[String],
    call: &IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Value, StdError> {
    match Command::new(command).args(args).output() {
        Ok(output) => match output.status.code() {
            Some(code) => Ok(Value::result_ok(call.record(
                s::STD_PROC_PROCESS_OUTPUT,
                vec![
                    Value::Integer(i64::from(code)),
                    Value::Str(String::from_utf8_lossy(&output.stdout).into_owned().into()),
                    Value::Str(String::from_utf8_lossy(&output.stderr).into_owned().into()),
                ],
                location,
            )?)),
            None => Ok(Value::result_error(Value::Str(
                "process terminated without an exit code".into(),
            ))),
        },
        Err(error) => Ok(Value::result_error(Value::Str(error.to_string().into()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn run_proc(stack: &mut Vec<Value>) {
        crate::execute_test_intrinsic(Intrinsic::Proc(ProcIntrinsic::Run), stack, test_location())
            .unwrap();
    }

    fn run_capture(stack: &mut Vec<Value>) {
        crate::execute_test_intrinsic(
            Intrinsic::Proc(ProcIntrinsic::RunCapture),
            stack,
            test_location(),
        )
        .unwrap();
    }

    #[test]
    fn run_returns_exit_code_for_successful_process() {
        let (command, args) = successful_process_fixture();
        let mut stack = vec![Value::Str(command.into()), Value::Array(args.into())];

        run_proc(&mut stack);

        assert_eq!(stack, vec![Value::result_ok(Value::Integer(0))]);
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
            Value::Array(Vec::new().into()),
        ];

        run_proc(&mut stack);

        assert!(matches!(stack[0], Value::ResultError(_)));
    }

    #[test]
    fn current_executable_returns_an_existing_file() {
        let Value::ResultOk(path) = current_executable() else {
            panic!("current executable lookup must succeed");
        };
        let Value::Str(path) = path.into_inner() else {
            panic!("current executable must be a string");
        };

        assert!(std::path::Path::new(path.as_ref()).is_file(), "{path}");
    }

    #[test]
    fn run_capture_returns_stdout_and_stderr_for_successful_process() {
        let (command, args) = capture_process_fixture(0);
        let mut stack = vec![Value::Str(command.into()), Value::Array(args.into())];

        run_capture(&mut stack);

        assert_capture(&stack[0], 0, "captured stdout", "captured stderr");
    }

    #[test]
    fn run_capture_preserves_non_zero_exit_code_and_output() {
        let (command, args) = capture_process_fixture(7);
        let mut stack = vec![Value::Str(command.into()), Value::Array(args.into())];

        run_capture(&mut stack);

        assert_capture(&stack[0], 7, "captured stdout", "captured stderr");
    }

    #[test]
    fn run_capture_returns_error_for_missing_command() {
        let mut stack = vec![
            Value::Str("__fpas_proc_capture_missing_command_f4c1a7d2__".into()),
            Value::Array(Vec::new().into()),
        ];

        run_capture(&mut stack);

        assert!(matches!(stack[0], Value::ResultError(_)));
    }

    fn assert_capture(value: &Value, exit_code: i64, stdout: &str, stderr: &str) {
        let Value::ResultOk(output) = value else {
            panic!("capture must return Ok");
        };
        let Value::Record(output) = output.as_ref() else {
            panic!("capture output must be a record");
        };
        assert_eq!(output.body().layout.type_name, s::STD_PROC_PROCESS_OUTPUT);
        assert_eq!(
            output.body().values,
            [
                Value::Integer(exit_code),
                Value::Str(stdout.into()),
                Value::Str(stderr.into()),
            ]
        );
    }

    #[cfg(windows)]
    fn capture_process_fixture(exit_code: i64) -> (String, Vec<Value>) {
        (
            "cmd".into(),
            vec![
                Value::Str("/C".into()),
                Value::Str(
                    format!(
                        "<nul set /p =captured stdout&1>&2 <nul set /p =captured stderr&exit /b {exit_code}"
                    )
                    .into(),
                ),
            ],
        )
    }

    #[cfg(not(windows))]
    fn capture_process_fixture(exit_code: i64) -> (String, Vec<Value>) {
        (
            "sh".into(),
            vec![
                Value::Str("-c".into()),
                Value::Str(
                    (format!(
                        "printf 'captured stdout'; printf 'captured stderr' >&2; exit {exit_code}"
                    ))
                    .into(),
                ),
            ],
        )
    }
}
