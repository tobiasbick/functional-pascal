//! Bounded native socket waits release pool workers and the all-stop debugger driver.
//!
//! Documentation: `docs/pascal/std/network/net.md`.

use super::Failure;
use super::connections::closing::Closing;
use super::request::{self, Operation, Output};
use crate::execution::vm::{VmError, hosted::HostedState, tasks::TaskSuspension, worker::Worker};
use std::task::Poll;

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
    work: Work,
}

enum Work {
    Native(Job),
    Closing(Closing),
}

struct Job {
    permit: Option<Permit>,
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
                let result = operation.run(&state, cancelled);
                // Receiver ownership lasts through cancellation and native-thread join.
                let _ = sender.send(result);
            })
            .map_err(|error| Failure::io(&error, "Could not start native network worker".into()))?;
        Ok(Self {
            permit: Some(permit),
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
        let Some(permit) = self.permit.take() else {
            return;
        };
        let state = permit.0.clone();
        let thread = self.thread.take();
        let (_, empty) = mpsc::channel();
        let receiver = std::mem::replace(
            self.receiver.get_mut().unwrap_or_else(|p| p.into_inner()),
            empty,
        );
        super::super::native::retire(&state, move || {
            if let Some(thread) = thread {
                let _ = thread.join();
            }
            // Resources not delivered to FPAS still belong to this wait.
            match receiver.try_recv() {
                Ok(Ok(Output::Connection(handle))) => {
                    let _ = permit.0.network_connections.abort(handle);
                }
                Ok(Ok(Output::Listener(handle))) => {
                    let _ = permit.0.network_listeners.close(handle);
                }
                _ => {}
            }
            drop(receiver);
            drop(permit);
        });
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
        if operation == Op::Close {
            super::arguments::require_count(self, arguments, 1)?;
            let handle = super::arguments::connection(self, &arguments[0])?;
            let resources = vec![handle.to_string()];
            let location = self.intrinsic_location();
            return match self.hosted.network_connections.begin_close(handle) {
                Ok(closing) => {
                    self.suspend_net_work(operation, resources, destination, Work::Closing(closing))
                }
                Err(error) => Ok(Some(Value::result_error(error.into_value(
                    self.layouts.as_ref(),
                    &Intrinsic::Net(operation).debugger_name(),
                    &resources,
                    location,
                )?))),
            };
        }
        let request::Prepared {
            operation: prepared,
            resources,
            token,
        } = request::prepare(self, operation, arguments)?;
        let location = self.intrinsic_location();
        let job = prepared.and_then(|operation| {
            if let Some(token) = token {
                let cancelled = self
                    .hosted
                    .cancellations
                    .is_cancelled(token)
                    .map_err(|error| Failure::argument("Token", error))?;
                if matches!(operation, Operation::Accept(_) | Operation::Connect(_)) && cancelled {
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
        self.suspend_net_work(operation, resources, destination, Work::Native(job))
    }

    fn suspend_net_work(
        &mut self,
        operation: Op,
        resources: Vec<String>,
        destination: Option<Register>,
        work: Work,
    ) -> Result<Option<Value>, VmError> {
        self.task_suspension = Some(TaskSuspension::Net(Box::new(Wait {
            operation,
            resources,
            destination,
            location: self.intrinsic_location(),
            work,
        })));
        self.suspend_requested = true;
        let _ = self.poll_task_suspension()?;
        Ok(None)
    }

    /// Resume a socket operation at its original instruction and release native ownership on finish.
    pub(in crate::execution::vm) fn poll_net_wait(
        &mut self,
        mut wait: Wait,
    ) -> Result<bool, VmError> {
        let outcome = match &mut wait.work {
            Work::Native(job) => job.poll(),
            Work::Closing(closing) => match closing.poll() {
                Poll::Ready(result) => Some(result.map(|()| Output::Done)),
                Poll::Pending => None,
            },
        };
        let Some(outcome) = outcome else {
            self.task_suspension = Some(TaskSuspension::Net(Box::new(wait)));
            return Ok(false);
        };
        let value = match outcome {
            Ok(Output::Connection(id) | Output::Listener(id)) => {
                Value::result_ok(Value::OpaqueHandle(id))
            }
            Ok(Output::Bytes(bytes)) => Value::result_ok(Value::Array(
                bytes
                    .into_iter()
                    .map(|b| Value::Integer(i64::from(b)))
                    .collect(),
            )),
            Ok(Output::Count(count)) => Value::result_ok(Value::Integer(count as i64)),
            Ok(Output::Done) => Value::result_ok(Value::Boolean(true)),
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
