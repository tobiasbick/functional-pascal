//! Supervise the leader, owned process group and all stdio through completion.

use std::io;
use std::process::Stdio;
use std::time::{Duration, Instant};

use super::failure::{Failure, Kind};
use super::options::Request;
use super::pipes::Pipes;
use super::registry::Processes;

pub(super) struct Output {
    pub code: i64,
    pub stdout: String,
    pub stderr: String,
}

impl Processes {
    pub(super) fn run(
        &self,
        request: &Request<'_>,
        receiver: Option<fpas_std::ProgramStderr>,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<Output, Failure> {
        request.validate()?;
        let reservation = self.reserve()?;
        let process = &reservation.process;
        let start = Instant::now();
        let deadline = request
            .options
            .timeout
            .map(|timeout| {
                start.checked_add(timeout).ok_or_else(|| {
                    Failure::argument("TimeoutMillis", "Timeout exceeds the monotonic clock range")
                })
            })
            .transpose()?;
        if process.stopped() || cancelled() {
            return Err(Failure::new(Kind::Cancelled));
        }
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return Err(Failure::new(Kind::Timeout));
        }
        let mut command = request.command();
        command.stdin(if request.inherit_input {
            Stdio::inherit()
        } else if request.options.input.is_empty() {
            Stdio::null()
        } else {
            Stdio::piped()
        });
        command.stdout(if request.capture {
            Stdio::piped()
        } else {
            Stdio::inherit()
        });
        command.stderr(if request.capture || receiver.is_some() {
            Stdio::piped()
        } else {
            Stdio::inherit()
        });
        process
            .spawn(command)
            .map_err(|error| Failure::new(Kind::Start(error)))?;
        let deferred = if !request.inherit_input {
            receiver.clone()
        } else {
            None
        };
        let mut pipes = {
            let mut child = process
                .child
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let child = child.as_mut().ok_or_else(|| {
                Failure::new(Kind::Execution(io::Error::other("Process ownership lost")))
            })?;
            Pipes::new(
                child,
                request.options.maximum,
                if request.inherit_input {
                    receiver
                } else {
                    None
                },
            )
            .map_err(|error| Failure::new(Kind::Execution(error)))?
        };
        let outcome = loop {
            if process.stopped() || cancelled() {
                break Err(Kind::Cancelled);
            }
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                break Err(Kind::Timeout);
            }
            let status = match process.poll() {
                Ok(status) => status,
                Err(error) => break Err(Kind::Wait(error)),
            };
            if process.stopped() || cancelled() {
                break Err(Kind::Cancelled);
            }
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                break Err(Kind::Timeout);
            }
            if let Some(status) = status
                && pipes.complete()
            {
                break status.code().map(i64::from).ok_or_else(|| {
                    Kind::Wait(io::Error::other("Process terminated without an exit code"))
                });
            }
            if process.stopped() || cancelled() {
                break Err(Kind::Cancelled);
            }
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                break Err(Kind::Timeout);
            }
            let progress = match pipes.poll(request.options.input.as_bytes()) {
                Ok(progress) => progress,
                Err(error) => break Err(error),
            };
            let status = match process.poll() {
                Ok(status) => status,
                Err(error) => break Err(Kind::Wait(error)),
            };
            if process.stopped() || cancelled() {
                break Err(Kind::Cancelled);
            }
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                break Err(Kind::Timeout);
            }
            if let Some(status) = status
                && pipes.complete()
            {
                break status.code().map(i64::from).ok_or_else(|| {
                    Kind::Wait(io::Error::other("Process terminated without an exit code"))
                });
            }
            if !progress {
                std::thread::sleep(Duration::from_millis(1));
            }
        };
        // Group cleanup also handles descendants that retain pipes after leader exit.
        let terminated = process.terminate();
        if outcome.is_err() {
            pipes.drain_after_abort();
        }
        pipes.close();
        let reaped = process.reap();
        let (stdout, stderr) = pipes.output();
        let outcome = outcome
            .and_then(|code| terminated.map_err(Kind::Execution).map(|()| code))
            .and_then(|code| reaped.map_err(Kind::Wait).map(|_| code));
        // Controlled calls deliver JSON stderr only after managed child cleanup and deadline decisions.
        if let Some(receiver) = deferred {
            for line in stderr.split_inclusive('\n') {
                receiver(line.trim_end_matches('\n').trim_end_matches('\r'));
            }
        }
        match outcome {
            Ok(code) => Ok(Output {
                code,
                stdout,
                stderr,
            }),
            Err(kind) => Err(Failure {
                kind,
                stdout,
                stderr,
            }),
        }
    }
}
