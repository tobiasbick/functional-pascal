//! Debugger socket waits release the all-stop driver while native I/O waits for a peer.
//!
//! Documentation: `docs/pascal/std/network/net.md`.

use super::{Failure, arguments as args};
use crate::execution::vm::{VmError, hosted::HostedState, tasks::TaskSuspension, worker::Worker};

#[cfg(test)]
#[path = "wait_tests.rs"]
mod tests;
use crate::{Intrinsic, NetIntrinsic as Op, Register, SourceLocation, Value};
use std::{
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::JoinHandle,
};

enum Operation {
    Accept(u64),
    Read(u64, i64),
    Write(u64, Vec<u8>),
}

enum Output {
    Connection(u64),
    Bytes(Vec<u8>),
    Count(usize),
}

/// Native-only socket result and its owned worker; contains no FPAS roots or callbacks.
pub(in crate::execution::vm) struct Wait {
    operation: Op,
    resource: String,
    destination: Option<Register>,
    location: SourceLocation,
    job: Job,
}

struct Job {
    hosted: Arc<HostedState>,
    stop: Arc<AtomicBool>,
    receiver: Mutex<mpsc::Receiver<Result<Output, Failure>>>,
    thread: Option<JoinHandle<()>>,
}

impl Job {
    fn start(
        hosted: Arc<HostedState>,
        operation: Operation,
        token: Option<u64>,
    ) -> Result<Self, Failure> {
        let stop = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = mpsc::channel();
        let state = Arc::clone(&hosted);
        let stopped = Arc::clone(&stop);
        let thread = std::thread::Builder::new()
            .name("fpas-debug-network".into())
            .spawn(move || {
                let cancelled = || {
                    stopped.load(Ordering::Acquire)
                        || token
                            .is_some_and(|id| state.cancellations.is_cancelled(id).unwrap_or(true))
                };
                let result = match operation {
                    Operation::Accept(handle) => state
                        .network_listeners
                        .accept_with_cancellation(handle, cancelled)
                        .and_then(|transport| {
                            if cancelled() {
                                Err(Failure::Cancelled)
                            } else {
                                state
                                    .network_connections
                                    .insert_accepted(transport)
                                    .map(Output::Connection)
                            }
                        }),
                    Operation::Read(handle, count) => state
                        .network_connections
                        .read_with_cancellation(handle, count, cancelled)
                        .map(Output::Bytes),
                    Operation::Write(handle, bytes) => state
                        .network_connections
                        .write_with_cancellation(handle, &bytes, cancelled)
                        .map(Output::Count),
                };
                // Receiver ownership lasts through join, including a debugger stop or disconnect.
                let _ = sender.send(result);
            })
            .map_err(|error| {
                Failure::io(&error, "Could not start debugger network worker".into())
            })?;
        Ok(Self {
            hosted,
            stop,
            receiver: Mutex::new(receiver),
            thread: Some(thread),
        })
    }

    fn poll(&self) -> Option<Result<Output, Failure>> {
        match self
            .receiver
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .try_recv()
        {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(Failure::io(
                &io::Error::other("Network worker exited"),
                "Debugger network worker exited without a result".into(),
            ))),
        }
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        // An accepted connection not delivered to FPAS still belongs to this abandoned wait.
        if let Ok(Ok(Output::Connection(handle))) = self
            .receiver
            .get_mut()
            .unwrap_or_else(|p| p.into_inner())
            .try_recv()
        {
            let _ = self.hosted.network_connections.close(handle);
        }
    }
}

impl Worker {
    /// Start socket I/O without blocking the all-stop scheduler; ordinary pool calls stay synchronous.
    pub(in crate::execution::vm) fn debug_net_wait(
        &mut self,
        operation: Op,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Value>, VmError> {
        let cancellable = matches!(
            operation,
            Op::AcceptWithCancellation | Op::ReadWithCancellation | Op::WriteWithCancellation
        );
        let accept = matches!(operation, Op::Accept | Op::AcceptWithCancellation);
        let count = if accept { 1 } else { 2 };
        args::require_count(self, arguments, count + usize::from(cancellable))?;
        let handle = if accept {
            args::listener(self, &arguments[0])?
        } else {
            args::connection(self, &arguments[0])?
        };
        let token = if cancellable {
            Some(args::cancellation_token(self, &arguments[count])?)
        } else {
            None
        };
        let location = self.intrinsic_location();
        let resource = handle.to_string();
        let prepared = if accept {
            Ok(Operation::Accept(handle))
        } else if matches!(operation, Op::Read | Op::ReadWithCancellation) {
            Ok(Operation::Read(
                handle,
                args::integer(self, &arguments[1], "MaxBytes")?,
            ))
        } else {
            args::bytes(self, &arguments[1])?.map(|data| Operation::Write(handle, data))
        };
        let job = prepared.and_then(|operation| {
            if let Some(token) = token {
                let cancelled = self
                    .hosted
                    .cancellations
                    .is_cancelled(token)
                    .map_err(|error| Failure::argument("Token", error))?;
                if accept && cancelled {
                    return Err(Failure::Cancelled);
                }
            }
            Job::start(Arc::clone(&self.hosted), operation, token)
        });
        let job = match job {
            Ok(job) => job,
            Err(error) => {
                return Ok(Some(Value::result_error(error.into_value(
                    self.layouts.as_ref(),
                    &Intrinsic::Net(operation).debugger_name(),
                    &[resource],
                    location,
                )?)));
            }
        };
        self.task_suspension = Some(TaskSuspension::Net(Box::new(Wait {
            operation,
            resource,
            destination,
            location,
            job,
        })));
        self.suspend_requested = true;
        let _ = self.poll_task_suspension()?;
        Ok(None)
    }

    /// Resume a socket operation at its original instruction and release native ownership on finish.
    pub(in crate::execution::vm) fn poll_net_wait(&mut self, wait: Wait) -> Result<bool, VmError> {
        let Some(outcome) = wait.job.poll() else {
            self.task_suspension = Some(TaskSuspension::Net(Box::new(wait)));
            return Ok(false);
        };
        let value = match outcome {
            Ok(Output::Connection(id)) => Value::result_ok(Value::OpaqueHandle(id)),
            Ok(Output::Bytes(bytes)) => Value::result_ok(Value::Array(
                bytes
                    .into_iter()
                    .map(|b| Value::Integer(i64::from(b)))
                    .collect(),
            )),
            Ok(Output::Count(count)) => Value::result_ok(Value::Integer(count as i64)),
            Err(error) => Value::result_error(error.into_value(
                self.layouts.as_ref(),
                &Intrinsic::Net(wait.operation).debugger_name(),
                std::slice::from_ref(&wait.resource),
                wait.location,
            )?),
        };
        if let Some(destination) = wait.destination {
            self.write(destination, value)?;
        }
        Ok(true)
    }
}
