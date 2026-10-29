//! Bounded, joined native file calls retain no FPAS values on the I/O thread.

use super::{
    call::{Call, Output},
    failure::Failure,
};
use crate::execution::vm::{VmError, hosted::HostedState, tasks::TaskSuspension, worker::Worker};
use crate::{FsIntrinsic, Register, SourceLocation, Value};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::JoinHandle,
};

#[cfg(test)]
mod tests;

const MAX_JOBS: usize = 64;

struct Permit(Arc<HostedState>);

impl Permit {
    fn acquire(hosted: Arc<HostedState>) -> Result<Self, Failure> {
        hosted
            .file_jobs
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < MAX_JOBS).then_some(count + 1)
            })
            .map_err(|_| Failure::Limit(MAX_JOBS as i64))?;
        Ok(Self(hosted))
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        self.0.file_jobs.fetch_sub(1, Ordering::AcqRel);
    }
}

struct Job {
    permit: Option<Permit>,
    stop: Arc<AtomicBool>,
    receiver: Mutex<mpsc::Receiver<Result<Output, Failure>>>,
    thread: Option<JoinHandle<()>>,
}

impl Job {
    #[cfg(test)]
    fn start(hosted: Arc<HostedState>, call: Call) -> Result<Self, Failure> {
        let permit = Permit::acquire(hosted)?;
        Self::start_reserved(permit, call)
    }

    fn start_reserved(permit: Permit, call: Call) -> Result<Self, Failure> {
        let state = Arc::clone(&permit.0);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&stop);
        let (sender, receiver) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("fpas-file".into())
            .spawn(move || {
                let _ = sender.send(call.run(&state, || stopped.load(Ordering::Acquire)));
            })
            .map_err(|error| Failure::io("", error))?;
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
                "",
                std::io::Error::other("Native file worker exited without a result"),
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
            // An opened file becomes the program's responsibility only after delivery.
            if let Ok(Ok(Output::Handle(handle))) = receiver.try_recv() {
                let _ = permit.0.files.close(handle, || false);
            }
            drop(receiver);
            drop(permit);
        });
    }
}

pub(in crate::execution::vm) struct Wait {
    state: State,
    operation: FsIntrinsic,
    location: SourceLocation,
    destination: Option<Register>,
}

enum State {
    Queued(Call),
    Running(Job),
}

impl Worker {
    pub(in crate::execution::vm) fn start_fs_wait(
        &mut self,
        operation: FsIntrinsic,
        values: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Value>, VmError> {
        let location = self.intrinsic_location();
        let state =
            Call::prepare(self, operation, values)?.and_then(|call| {
                match Permit::acquire(Arc::clone(&self.hosted)) {
                    Ok(permit) => Job::start_reserved(permit, call).map(State::Running),
                    Err(Failure::Limit(_)) if crate::stdlib::fs::native::infallible(operation) => {
                        Ok(State::Queued(call))
                    }
                    Err(error) => Err(error),
                }
            });
        let state = match state {
            Ok(state) => state,
            Err(error) => return self.fs_result(operation, Err(error), location).map(Some),
        };
        self.task_suspension = Some(TaskSuspension::Fs(Box::new(Wait {
            state,
            operation,
            location,
            destination,
        })));
        self.suspend_requested = true;
        let _ = self.poll_task_suspension()?;
        Ok(None)
    }

    pub(in crate::execution::vm) fn poll_fs_wait(
        &mut self,
        mut wait: Wait,
    ) -> Result<bool, VmError> {
        if let State::Queued(call) = wait.state {
            let result = match Permit::acquire(Arc::clone(&self.hosted)) {
                Ok(permit) => Job::start_reserved(permit, call),
                Err(Failure::Limit(_)) => {
                    wait.state = State::Queued(call);
                    self.task_suspension = Some(TaskSuspension::Fs(Box::new(wait)));
                    return Ok(false);
                }
                Err(error) => {
                    return self.finish_fs_wait(
                        wait.operation,
                        wait.destination,
                        wait.location,
                        Err(error),
                    );
                }
            };
            match result {
                Ok(job) => wait.state = State::Running(job),
                Err(error) => {
                    return self.finish_fs_wait(
                        wait.operation,
                        wait.destination,
                        wait.location,
                        Err(error),
                    );
                }
            }
        }
        let State::Running(job) = &wait.state else {
            unreachable!("queued admission was handled above")
        };
        let Some(outcome) = job.poll() else {
            self.task_suspension = Some(TaskSuspension::Fs(Box::new(wait)));
            return Ok(false);
        };
        self.finish_fs_wait(wait.operation, wait.destination, wait.location, outcome)
    }

    fn finish_fs_wait(
        &mut self,
        operation: FsIntrinsic,
        destination: Option<Register>,
        location: SourceLocation,
        outcome: Result<Output, Failure>,
    ) -> Result<bool, VmError> {
        let value = self.fs_result(operation, outcome, location)?;
        if let Some(destination) = destination {
            self.write(destination, value)?;
        }
        Ok(true)
    }
}
