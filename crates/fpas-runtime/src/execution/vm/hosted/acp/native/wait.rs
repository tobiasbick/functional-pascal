//! Bounded native ACP jobs own their requests, late results and cleanup until delivery.

use super::super::wait::Wait as AcpWait;
use super::{call::Call, output::Output};
use crate::execution::vm::{
    VmError,
    hosted::{HostedState, native},
    worker::Worker,
};
use crate::{AcpIntrinsic as Op, Register, SourceLocation, Value};
use fpas_acp::connection::Fault;
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
impl Permit {
    fn reserve(state: Arc<HostedState>) -> Result<Self, Fault> {
        state
            .acp_jobs
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < LIMIT).then_some(count + 1)
            })
            .map_err(|_| Fault::LimitExceeded("NativeJobs", LIMIT))?;
        Ok(Self(state))
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.acp_jobs.fetch_sub(1, Ordering::AcqRel);
    }
}
pub(in crate::execution::vm) struct Wait {
    job: Job,
    operation: Op,
    destination: Option<Register>,
    location: SourceLocation,
}
struct Job {
    permit: Option<Permit>,
    stop: Arc<AtomicBool>,
    receiver: Option<Mutex<mpsc::Receiver<Result<Output, Fault>>>>,
    thread: Option<JoinHandle<()>>,
}
impl Job {
    fn start(state: Arc<HostedState>, call: Call) -> Result<Self, Fault> {
        let permit = Permit::reserve(state.clone())?;
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let (send, receiver) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("fpas-acp-host".into())
            .spawn(move || {
                native::defer_joins(|| {
                    let _ = send.send(call.run(&state, || stopped.load(Ordering::Acquire)));
                });
            })
            .map_err(|_| Fault::Host)?;
        Ok(Self {
            permit: Some(permit),
            stop,
            receiver: Some(Mutex::new(receiver)),
            thread: Some(thread),
        })
    }
    fn poll(&self) -> Option<Result<Output, Fault>> {
        let Some(receiver) = &self.receiver else {
            unreachable!("live ACP job")
        };
        match receiver
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .try_recv()
        {
            Ok(value) => Some(value),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(Fault::Host)),
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
        let receiver = self.receiver.take();
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
    pub(in crate::execution::vm) fn start_acp_native(
        &mut self,
        operation: Op,
        values: &[Value],
        destination: Option<Register>,
    ) -> Result<(), VmError> {
        let location = self.intrinsic_location();
        let job = Call::prepare(self, operation, values)
            .and_then(|call| Job::start(self.hosted.clone(), call));
        let job = match job {
            Ok(job) => job,
            Err(error) => {
                let value = self.acp_result(&operation.source_name(), Err(error), location)?;
                if let Some(destination) = destination {
                    self.write(destination, value)?;
                }
                return Ok(());
            }
        };
        self.suspend_acp_wait(AcpWait::Native(Wait {
            job,
            operation,
            destination,
            location,
        }))?;
        Ok(())
    }
    pub(in crate::execution::vm) fn poll_acp_native(
        &mut self,
        wait: Wait,
    ) -> Result<bool, VmError> {
        let Some(output) = wait.job.poll() else {
            self.task_suspension = Some(crate::execution::vm::tasks::TaskSuspension::Acp(
                Box::new(AcpWait::Native(wait)),
            ));
            return Ok(false);
        };
        let result = output.and_then(|output| output.into_value(self, wait.location));
        let value = self.acp_result(&wait.operation.source_name(), result, wait.location)?;
        if let Some(destination) = wait.destination {
            self.write(destination, value)?;
        }
        Ok(true)
    }
}
