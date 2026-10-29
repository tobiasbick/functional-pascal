//! Bounded native socket waits release pool workers and the all-stop debugger driver.
//!
//! Documentation: `docs/pascal/std/network/net.md`.

use super::connections::{ConnectMode, ConnectRequest};
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
    Connect(ConnectRequest),
    Accept(u64),
    Read(u64, i64),
    Write(u64, Vec<u8>),
}

enum Output {
    Connection(u64),
    Bytes(Vec<u8>),
    Count(usize),
}

pub(super) const MAX_PENDING_JOBS: usize = 64;

/// Keeps the native thread and any undelivered result within one per-VM budget.
struct Permit(Arc<HostedState>);

impl Permit {
    fn acquire(hosted: Arc<HostedState>) -> Result<Self, Failure> {
        hosted
            .network_jobs
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < MAX_PENDING_JOBS).then_some(count + 1)
            })
            .map_err(|_| {
                Failure::io(
                    &io::Error::other("Pending network operation limit reached"),
                    format!(
                        "At most {MAX_PENDING_JOBS} native socket operations may be pending per VM"
                    ),
                )
            })?;
        Ok(Self(hosted))
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        self.0.network_jobs.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Saved socket operation; FPAS frames and callbacks remain in the owning task state.
pub(in crate::execution::vm) struct Wait {
    operation: Op,
    resources: Vec<String>,
    destination: Option<Register>,
    location: SourceLocation,
    job: Job,
}

struct Job {
    permit: Permit,
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
        let permit = Permit::acquire(hosted)?;
        let stop = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = mpsc::channel();
        let state = Arc::clone(&permit.0);
        let stopped = Arc::clone(&stop);
        let thread = std::thread::Builder::new()
            .name("fpas-network".into())
            .spawn(move || {
                let cancelled = || {
                    stopped.load(Ordering::Acquire)
                        || token
                            .is_some_and(|id| state.cancellations.is_cancelled(id).unwrap_or(true))
                };
                let result = match operation {
                    Operation::Connect(request) => state
                        .network_connections
                        .connect_prepared(&request, cancelled)
                        .map(Output::Connection),
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
                // Receiver ownership lasts through cancellation and native-thread join.
                let _ = sender.send(result);
            })
            .map_err(|error| Failure::io(&error, "Could not start native network worker".into()))?;
        Ok(Self {
            permit,
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
                "Native network worker exited without a result".into(),
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
        // An accepted or outgoing connection not delivered to FPAS belongs to this wait.
        if let Ok(Ok(Output::Connection(handle))) = self
            .receiver
            .get_mut()
            .unwrap_or_else(|p| p.into_inner())
            .try_recv()
        {
            let _ = self.permit.0.network_connections.close(handle);
        }
    }
}

impl Worker {
    /// Start socket I/O without blocking its pool worker or the all-stop scheduler.
    pub(in crate::execution::vm) fn start_net_wait(
        &mut self,
        operation: Op,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Value>, VmError> {
        let cancellable = matches!(
            operation,
            Op::AcceptWithCancellation
                | Op::ReadWithCancellation
                | Op::WriteWithCancellation
                | Op::ConnectWithCancellation
                | Op::ConnectTlsWithCancellation
        );
        let connect = matches!(
            operation,
            Op::Connect
                | Op::ConnectTls
                | Op::ConnectWithCancellation
                | Op::ConnectTlsWithCancellation
        );
        let accept = matches!(operation, Op::Accept | Op::AcceptWithCancellation);
        let count = if connect {
            3
        } else if accept {
            1
        } else {
            2
        };
        args::require_count(self, arguments, count + usize::from(cancellable))?;
        let token = if cancellable {
            Some(args::cancellation_token(self, &arguments[count])?)
        } else {
            None
        };
        let location = self.intrinsic_location();
        let (prepared, resources) = if connect {
            let host = args::string(self, &arguments[0], "Host")?;
            let port = args::integer(self, &arguments[1], "Port")?;
            let timeout = args::integer(self, &arguments[2], "TimeoutMillis")?;
            let mode = if matches!(operation, Op::ConnectTls | Op::ConnectTlsWithCancellation) {
                ConnectMode::Tls
            } else {
                ConnectMode::Tcp
            };
            (
                ConnectRequest::new(host, port, timeout, mode).map(Operation::Connect),
                vec![host.to_owned(), port.to_string()],
            )
        } else {
            let handle = if accept {
                args::listener(self, &arguments[0])?
            } else {
                args::connection(self, &arguments[0])?
            };
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
            (prepared, vec![handle.to_string()])
        };
        let job = prepared.and_then(|operation| {
            if let Some(token) = token {
                let cancelled = self
                    .hosted
                    .cancellations
                    .is_cancelled(token)
                    .map_err(|error| Failure::argument("Token", error))?;
                if (accept || connect) && cancelled {
                    return Err(Failure::Cancelled);
                }
            }
            if let Operation::Read(_, count) = &operation {
                super::connections::read_size(*count)?;
            }
            Job::start(Arc::clone(&self.hosted), operation, token)
        });
        let job = match job {
            Ok(job) => job,
            Err(error) => {
                return Ok(Some(Value::result_error(error.into_value(
                    self.layouts.as_ref(),
                    &Intrinsic::Net(operation).debugger_name(),
                    &resources,
                    location,
                )?)));
            }
        };
        self.task_suspension = Some(TaskSuspension::Net(Box::new(Wait {
            operation,
            resources,
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
                &wait.resources,
                wait.location,
            )?),
        };
        if let Some(destination) = wait.destination {
            self.write(destination, value)?;
        }
        Ok(true)
    }
}
