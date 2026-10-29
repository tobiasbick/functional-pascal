//! Supervise the leader, owned process group and all stdio through completion.

use std::io;
use std::process::Stdio;
use std::task::Poll;
use std::time::{Duration, Instant};

use super::failure::{Failure, Kind};
use super::options::Request;
use super::pipes::Pipes;
use super::registry::{Processes, Reservation};

mod state;

pub(super) struct Output {
    pub code: i64,
    pub stdout: String,
    pub stderr: String,
}

/// Owned native state for a process call, movable between task execution lanes.
pub(super) struct Execution {
    // Declared first: abandonment terminates and reaps before releasing pipe state.
    reservation: Option<Reservation>,
    pipes: Pipes,
    deadline: Option<Instant>,
    deferred: Option<crate::stdlib::ProgramStderr>,
    progress: bool,
}

impl Processes {
    pub(super) fn run(
        &self,
        request: &Request<'_>,
        receiver: Option<crate::stdlib::ProgramStderr>,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<Output, Failure> {
        let mut execution = self.start(request, receiver, &mut cancelled)?;
        loop {
            match execution.poll(request.options.input.as_bytes(), &mut cancelled) {
                Poll::Ready(outcome) => return outcome,
                Poll::Pending if !execution.made_progress() => {
                    std::thread::sleep(Duration::from_millis(1));
                }
                Poll::Pending => {}
            }
        }
    }

    pub(super) fn start(
        &self,
        request: &Request<'_>,
        receiver: Option<crate::stdlib::ProgramStderr>,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<Execution, Failure> {
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
        let pipes = {
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
        Ok(Execution {
            reservation: Some(reservation),
            pipes,
            deadline,
            deferred,
            progress: false,
        })
    }
}
