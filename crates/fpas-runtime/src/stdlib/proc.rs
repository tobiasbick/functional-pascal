//! Pure `Std.Os.Proc` defaults and executable-path lookup.
//!
//! **Documentation:** `docs/pascal/std/host/proc.md` (from the repository root).

mod options;

use crate::stdlib::error::StdError;
use crate::stdlib::intrinsic_args::IntrinsicCall;
use crate::{Intrinsic, ProcIntrinsic, SourceLocation, Value};
use std::env;
use std::sync::Arc;

/// Receives standard-error lines from hosted process calls.
///
/// Lines arrive without their line ending. A host installs a receiver when the
/// child's standard error must not reach the parent's stderr directly, for
/// example to wrap it as program-output events in a JSON diagnostic stream.
/// Legacy `Run` forwards live; `RunWithOptions` delivers bounded stderr after
/// child cleanup, outside its managed process deadline.
pub type ProgramStderr = Arc<dyn Fn(&str) + Send + Sync>;

/// Execute a `Std.Os.Proc` intrinsic and return `None` when another unit should handle it.
///
/// Process starts use the VM's owned process engine. This owner constructs
/// defaults and reads the executable path.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Proc(ProcIntrinsic::DefaultStartOptions) => {
            call.push(call.factory().record(
                "Std.Os.Proc.ProcessStartOptions",
                vec![Value::OptionNone, Value::Array(Vec::new().into())],
                location,
            )?);
        }
        Intrinsic::Proc(ProcIntrinsic::DefaultOptions) => {
            call.push(options::defaults(call.factory(), location)?);
        }
        Intrinsic::Proc(ProcIntrinsic::CurrentExecutable) => {
            call.push(current_executable(call.factory(), location)?);
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn current_executable(
    factory: &dyn crate::stdlib::AggregateFactory,
    location: SourceLocation,
) -> Result<Value, StdError> {
    match env::current_exe() {
        Ok(path) => Ok(Value::result_ok(Value::Str(
            path.to_string_lossy().into_owned().into(),
        ))),
        Err(error) => {
            let cause = crate::stdlib::native_errors::host(
                factory,
                "Std.Os.Proc.CurrentExecutable",
                &[],
                error.raw_os_error(),
                error.to_string(),
                location,
            )?;
            crate::stdlib::native_errors::value(
                factory,
                crate::stdlib::native_errors::NativeErrorFamily::Proc,
                3,
                vec![cause],
                location,
            )
            .map(Value::result_error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_executable_returns_an_existing_file() {
        let Value::ResultOk(path) = current_executable(
            &crate::stdlib::intrinsics::TEST_AGGREGATES,
            SourceLocation::new(1, 1),
        )
        .unwrap() else {
            panic!("current executable lookup must succeed");
        };
        let Value::Str(path) = path.into_inner() else {
            panic!("current executable must be a string");
        };

        assert!(std::path::Path::new(path.as_ref()).is_file(), "{path}");
    }
}
