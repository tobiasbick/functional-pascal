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
    permit: Permit,
    stop: Arc<AtomicBool>,
    receiver: Mutex<mpsc::Receiver<Result<Output, Failure>>>,
    thread: Option<JoinHandle<()>>,
}

impl Job {
    fn start(hosted: Arc<HostedState>, call: Call) -> Result<Self, Failure> {
        let permit = Permit::acquire(hosted)?;
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
                "",
                std::io::Error::other("Native file worker exited without a result"),
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
        // An opened file becomes the program's responsibility only after delivery.
        if let Ok(Ok(Output::Handle(handle))) = self
            .receiver
            .get_mut()
            .unwrap_or_else(|p| p.into_inner())
            .try_recv()
        {
            let _ = self.permit.0.files.close(handle, || false);
        }
    }
}

pub(in crate::execution::vm) struct Wait {
    job: Job,
    operation: FsIntrinsic,
    location: SourceLocation,
    destination: Option<Register>,
}

impl Worker {
    pub(in crate::execution::vm) fn start_fs_wait(
        &mut self,
        operation: FsIntrinsic,
        values: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Value>, VmError> {
        let location = self.intrinsic_location();
        let job = Call::prepare(self, operation, values)?
            .and_then(|call| Job::start(Arc::clone(&self.hosted), call));
        let job = match job {
            Ok(job) => job,
            Err(error) => return self.fs_result(operation, Err(error), location).map(Some),
        };
        self.task_suspension = Some(TaskSuspension::Fs(Box::new(Wait {
            job,
            operation,
            location,
            destination,
        })));
        self.suspend_requested = true;
        let _ = self.poll_task_suspension()?;
        Ok(None)
    }

    pub(in crate::execution::vm) fn poll_fs_wait(&mut self, wait: Wait) -> Result<bool, VmError> {
        let Some(outcome) = wait.job.poll() else {
            self.task_suspension = Some(TaskSuspension::Fs(Box::new(wait)));
            return Ok(false);
        };
        let value = self.fs_result(wait.operation, outcome, wait.location)?;
        if let Some(destination) = wait.destination {
            self.write(destination, value)?;
        }
        Ok(true)
    }
}
