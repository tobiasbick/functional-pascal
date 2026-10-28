//! Preserve the native status without changing the legacy integer view.

use std::process::ExitStatus;

use crate::{SourceLocation, Value};

pub(super) fn value(
    factory: &dyn crate::stdlib::AggregateFactory,
    status: ExitStatus,
    location: SourceLocation,
) -> Result<Value, crate::stdlib::StdError> {
    #[cfg(windows)]
    let code = status.code().map(|code| i64::from(code as u32));
    #[cfg(not(windows))]
    let code = status.code().map(i64::from);
    let code = code.map_or(Value::OptionNone, |code| {
        Value::option_some(Value::Integer(code))
    });
    #[cfg(unix)]
    let signal = {
        use std::os::unix::process::ExitStatusExt;
        status.signal().map_or(Value::OptionNone, |number| {
            let name = nix::sys::signal::Signal::try_from(number)
                .map(|signal| signal.as_str().to_owned())
                .unwrap_or_else(|_| number.to_string());
            Value::option_some(Value::Str(name.into()))
        })
    };
    #[cfg(not(unix))]
    let signal = Value::OptionNone;
    factory.record(
        "Std.Os.Proc.ProcessExitStatus",
        vec![code, signal],
        location,
    )
}
