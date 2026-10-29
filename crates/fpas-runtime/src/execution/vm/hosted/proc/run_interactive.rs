//! Interactive process execution reusing the common owned process registry.

use std::process::ExitStatus;
use std::time::{Duration, Instant};

use crate::{Intrinsic, ProcIntrinsic, SourceLocation, Value};

use super::arguments;
use super::failure::{Failure, Kind};
use super::options::Request;
use super::platform::Foreground;
use super::registry::Processes;
use crate::execution::InteractiveProcessTerminal;
use crate::execution::vm::{Vm, VmError, worker::Worker};

impl Vm {
    /// Install explicit child-process terminal support; ordinary VM constructors have none.
    pub fn set_interactive_process_terminal(&mut self, terminal: InteractiveProcessTerminal) {
        *self
            .hosted
            .process_terminal
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(terminal);
    }
}

impl Worker {
    pub(super) fn execute_run_interactive(
        &self,
        values: &[Value],
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        let outcome = arguments::parse_interactive(values).and_then(|request| {
            if let Some(token) = request.options.token {
                self.hosted
                    .cancellations
                    .is_cancelled(token)
                    .map_err(|error| Failure::argument("CancellationToken", error))?;
            }
            let terminal = self
                .hosted
                .process_terminal
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            self.hosted
                .processes
                .run_interactive(&request, terminal.as_ref(), || {
                    self.check_task_cancellation().is_err()
                        || self
                            .scheduler
                            .as_ref()
                            .is_some_and(|scheduler| scheduler.is_aborted())
                        || request.options.token.is_some_and(|token| {
                            self.hosted
                                .cancellations
                                .is_cancelled(token)
                                .unwrap_or(true)
                        })
                })
        });
        Ok(match outcome {
            Ok(status) => Value::result_ok(super::status::value(
                self.layouts.as_ref(),
                status,
                location,
            )?),
            Err(error) => Value::result_error(error.into_value(
                self.layouts.as_ref(),
                &Intrinsic::Proc(ProcIntrinsic::RunInteractive).debugger_name(),
                arguments::command_context(values),
                false,
                location,
            )?),
        })
    }
}

impl Processes {
    pub(super) fn run_interactive(
        &self,
        request: &Request<'_>,
        terminal: Option<&InteractiveProcessTerminal>,
        cancelled: impl FnMut() -> bool,
    ) -> Result<ExitStatus, Failure> {
        self.run_interactive_at(request, terminal, cancelled, Instant::now())
    }

    pub(super) fn run_interactive_at(
        &self,
        request: &Request<'_>,
        terminal: Option<&InteractiveProcessTerminal>,
        mut cancelled: impl FnMut() -> bool,
        started: Instant,
    ) -> Result<ExitStatus, Failure> {
        request.validate()?;
        let reservation = self.reserve()?;
        let process = &reservation.process;
        let deadline = request
            .options
            .timeout
            .map(|timeout| {
                started.checked_add(timeout).ok_or_else(|| {
                    Failure::argument("TimeoutMillis", "Timeout exceeds the monotonic clock range")
                })
            })
            .transpose()?;
        let check = |cancelled: &mut dyn FnMut() -> bool| {
            if process.stopped() || cancelled() {
                Err(Failure::new(Kind::Cancelled))
            } else if deadline.is_some_and(|end| Instant::now() >= end) {
                Err(Failure::new(Kind::Timeout))
            } else {
                Ok(())
            }
        };
        check(&mut cancelled)?;
        let terminal = terminal.ok_or_else(|| {
            Failure::argument("Terminal", "Host has no interactive process terminal")
        })?;
        let mut streams = terminal
            .open()
            .map_err(|error| Failure::new(Kind::Execution(error)))?;
        let mut foreground =
            Foreground::new(&mut streams).map_err(|error| Failure::new(Kind::Execution(error)))?;
        let mut command = request.command();
        command
            .stdin(streams.stdin)
            .stdout(streams.stdout)
            .stderr(streams.stderr);
        check(&mut cancelled)?;
        process
            .spawn(command)
            .map_err(|error| Failure::new(Kind::Start(error)))?;
        foreground
            .activate(
                process
                    .id()
                    .map_err(|error| Failure::new(Kind::Execution(error)))?,
            )
            .map_err(|error| Failure::new(Kind::Execution(error)))?;
        let outcome = loop {
            if process.stopped() || cancelled() {
                break Err(Kind::Cancelled);
            }
            let status = match process.poll() {
                Ok(status) => status,
                Err(error) => break Err(Kind::Wait(error)),
            };
            if process.stopped() || cancelled() {
                break Err(Kind::Cancelled);
            }
            if let Some(status) = status {
                break Ok(status);
            }
            if deadline.is_some_and(|end| Instant::now() >= end) {
                break Err(Kind::Timeout);
            }
            std::thread::sleep(Duration::from_millis(1));
        };
        let terminated = process.terminate();
        let reaped = process.reap();
        let restored = foreground.restore();
        outcome
            .and_then(|status| {
                terminated.map_err(Kind::Execution)?;
                reaped.map_err(Kind::Wait)?;
                restored.map_err(Kind::Execution)?;
                Ok(status)
            })
            .map_err(Failure::new)
    }
}
