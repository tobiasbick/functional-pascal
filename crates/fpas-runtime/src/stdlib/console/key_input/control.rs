//! Cancellation boundaries around bounded terminal readiness probes.

use super::{
    KeyInput,
    event_source::{CrosstermEventSource, TerminalEventSource},
};
use crate::{
    SourceLocation,
    stdlib::{ConsoleEvent, StdError},
};
use std::time::{Duration, Instant};

impl KeyInput {
    pub(crate) fn read_event_timeout_controlled(
        &mut self,
        milliseconds: i64,
        location: SourceLocation,
        cancelled: impl Fn() -> bool,
    ) -> Result<Option<ConsoleEvent>, StdError> {
        self.read_event_timeout_controlled_from(
            milliseconds,
            location,
            cancelled,
            &mut CrosstermEventSource::default(),
        )
    }

    fn read_event_timeout_controlled_from(
        &mut self,
        milliseconds: i64,
        location: SourceLocation,
        cancelled: impl Fn() -> bool,
        source: &mut impl TerminalEventSource,
    ) -> Result<Option<ConsoleEvent>, StdError> {
        check_cancelled(&cancelled, location)?;
        let duration = Duration::from_millis(milliseconds.max(0) as u64);
        let started = Instant::now();
        loop {
            check_cancelled(&cancelled, location)?;
            let left = duration.saturating_sub(started.elapsed());
            let slice = left.min(Duration::from_millis(10)).as_millis() as i64;
            let event = self.read_event_timeout_from(slice, location, source)?;
            if event.is_some()
                || !self.allows_os_events()
                || !self.raw_mode
                || started.elapsed() >= duration
            {
                return Ok(event);
            }
        }
    }
}

#[cfg(test)]
mod tests;
pub(super) fn check_cancelled(
    cancelled: impl Fn() -> bool,
    location: SourceLocation,
) -> Result<(), StdError> {
    if cancelled() {
        Err(crate::stdlib::error::std_runtime_error(
            fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED,
            "Console input was cancelled",
            "Start a new input operation if needed.",
            location,
        ))
    } else {
        Ok(())
    }
}

pub(super) fn input_error(
    context: &str,
    error: std::io::Error,
    location: SourceLocation,
) -> StdError {
    crate::stdlib::error::std_runtime_error(
        fpas_diagnostics::codes::RUNTIME_CONSOLE_INPUT_FAILURE,
        format!("{context}: {error}"),
        "Check terminal input availability and try again.",
        location,
    )
}
