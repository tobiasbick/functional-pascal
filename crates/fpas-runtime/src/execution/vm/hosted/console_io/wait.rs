//! Native console calls queue at capacity and keep result conversion on their VM task.

use super::{Request, request::Output};
use crate::execution::vm::{
    VmError,
    hosted::{HostedState, native},
    tasks::TaskSuspension,
    worker::Worker,
};
use crate::{Intrinsic, Register, SourceLocation, Value, stdlib::StdError};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::JoinHandle,
};

const LIMIT: usize = 64;
#[cfg(test)]
mod tests;
struct Permit(Arc<HostedState>);
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.console_jobs.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(in crate::execution::vm) struct Wait {
    state: State,
    operation: Intrinsic,
    destination: Option<Register>,
    location: SourceLocation,
}

enum State {
    Queued(Request),
    Running(Job),
}

struct Job {
    permit: Option<Permit>,
    stop: Arc<AtomicBool>,
    receiver: Option<Mutex<mpsc::Receiver<Result<Output, StdError>>>>,
    thread: Option<JoinHandle<()>>,
}

fn reserve(state: Arc<HostedState>) -> Option<Permit> {
    state
        .console_jobs
        .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
            (count < LIMIT).then_some(count + 1)
        })
        .ok()
        .map(|_| Permit(state))
}

fn host_error(location: SourceLocation, detail: impl std::fmt::Display) -> StdError {
    crate::stdlib::error::std_runtime_error(
        fpas_diagnostics::codes::RUNTIME_HOST_IO_FAILURE,
        format!("Native console worker failed: {detail}"),
        "Retry after earlier native I/O has finished.",
        location,
    )
}

impl Job {
    fn start(permit: Permit, call: Request) -> Result<Self, StdError> {
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let state = permit.0.clone();
        let location = call.location();
        let (send, receiver) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("fpas-console".into())
            .spawn(move || {
                native::defer_joins(|| {
                    let result = call.run(&state, || stopped.load(Ordering::Acquire));
                    drop(call);
                    let _ = send.send(result);
                })
            })
            .map_err(|error| host_error(location, error))?;
        Ok(Self {
            permit: Some(permit),
            stop,
            receiver: Some(Mutex::new(receiver)),
            thread: Some(thread),
        })
    }

    fn poll(&self, location: SourceLocation) -> Option<Result<Output, StdError>> {
        let Some(receiver) = &self.receiver else {
            unreachable!("live native console receiver")
        };
        match receiver
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .try_recv()
        {
            Ok(value) => Some(value),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err(host_error(location, "worker exited without a result")))
            }
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
        let receiver = self.receiver.take();
        let thread = self.thread.take();
        native::retire(&state, move || {
            if let Some(thread) = thread {
                let _ = thread.join();
            }
            drop(receiver);
            drop(permit);
        });
    }
}

impl Worker {
    pub(in crate::execution::vm) fn start_console_io(
        &mut self,
        operation: Intrinsic,
        values: &[Value],
        destination: Option<Register>,
    ) -> Result<(), VmError> {
        let location = self.intrinsic_location();
        let call = Request::prepare(self, operation, values, location)?;
        let state = match call.ready().then(|| reserve(self.hosted.clone())).flatten() {
            Some(permit) => State::Running(Job::start(permit, call)?),
            None => State::Queued(call),
        };
        self.task_suspension = Some(TaskSuspension::ConsoleIo(Box::new(Wait {
            state,
            operation,
            destination,
            location,
        })));
        self.suspend_requested = true;
        let _ = self.poll_task_suspension()?;
        Ok(())
    }

    pub(in crate::execution::vm) fn poll_console_io(
        &mut self,
        mut wait: Wait,
    ) -> Result<bool, VmError> {
        if let State::Queued(call) = wait.state {
            let Some(permit) = call.ready().then(|| reserve(self.hosted.clone())).flatten() else {
                wait.state = State::Queued(call);
                self.task_suspension = Some(TaskSuspension::ConsoleIo(Box::new(wait)));
                return Ok(false);
            };
            wait.state = State::Running(Job::start(permit, call)?);
        }
        let State::Running(job) = &wait.state else {
            unreachable!("queued console admission was handled")
        };
        let Some(outcome) = job.poll(wait.location) else {
            self.task_suspension = Some(TaskSuspension::ConsoleIo(Box::new(wait)));
            return Ok(false);
        };
        let result = outcome?.into_value(self, wait.location)?;
        self.store_intrinsic_result(wait.operation, wait.destination, result)?;
        Ok(true)
    }
}
