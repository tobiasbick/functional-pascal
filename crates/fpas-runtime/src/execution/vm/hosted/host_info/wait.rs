//! One admitted collector job per VM; queued tasks leave the worker pool free.
use super::super::{HostedState, native};
use crate::execution::vm::{VmError, tasks::TaskSuspension, worker::Worker};
use crate::{Intrinsic, Register, SourceLocation, SystemInfoIntrinsic as Op, Value};
use serde_json::Value as Json;
use std::sync::{Arc, Mutex, atomic::Ordering, mpsc};

struct Permit(Arc<HostedState>);
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.host_info.busy.store(false, Ordering::Release);
    }
}
fn reserve(state: Arc<HostedState>) -> Option<Permit> {
    state
        .host_info
        .busy
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .ok()
        .map(|_| Permit(state))
}
struct Job {
    permit: Option<Permit>,
    receiver: Option<Mutex<mpsc::Receiver<Result<Json, String>>>>,
}
impl Job {
    fn start(permit: Permit, operation: Op) -> Result<Self, String> {
        let receiver = permit.0.host_info.submit(operation)?;
        Ok(Self {
            permit: Some(permit),
            receiver: Some(Mutex::new(receiver)),
        })
    }
    fn poll(&self) -> Option<Result<Json, String>> {
        let receiver = self.receiver.as_ref()?;
        match receiver
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .try_recv()
        {
            Ok(value) => Some(value),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err("Host information worker exited without a result".into()))
            }
        }
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        let Some(permit) = self.permit.take() else {
            return;
        };
        let state = permit.0.clone();
        let receiver = self.receiver.take();
        native::retire(&state, move || {
            if let Some(receiver) = receiver {
                let _ = receiver
                    .into_inner()
                    .unwrap_or_else(|p| p.into_inner())
                    .recv();
            }
            drop(permit);
        });
    }
}
pub(in crate::execution::vm) struct Wait {
    job: Option<Job>,
    operation: Op,
    destination: Option<Register>,
    location: SourceLocation,
}
impl Worker {
    pub(in crate::execution::vm) fn start_host_info(
        &mut self,
        operation: Op,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<(), VmError> {
        self.require_host_info_arguments(arguments)?;
        self.task_suspension = Some(TaskSuspension::HostInfo(Box::new(Wait {
            job: None,
            operation,
            destination,
            location: self.intrinsic_location(),
        })));
        self.suspend_requested = true;
        let _ = self.poll_task_suspension()?;
        Ok(())
    }
    pub(in crate::execution::vm) fn poll_host_info(
        &mut self,
        mut wait: Wait,
    ) -> Result<bool, VmError> {
        if wait.job.is_none() {
            let Some(permit) = reserve(self.hosted.clone()) else {
                self.task_suspension = Some(TaskSuspension::HostInfo(Box::new(wait)));
                return Ok(false);
            };
            match Job::start(permit, wait.operation) {
                Ok(job) => wait.job = Some(job),
                Err(error) => {
                    let value = self.host_info_result(wait.operation, Err(error), wait.location)?;
                    self.store_intrinsic_result(
                        Intrinsic::SystemInfo(wait.operation),
                        wait.destination,
                        Some(value),
                    )?;
                    return Ok(true);
                }
            }
        }
        let Some(outcome) = wait.job.as_ref().and_then(Job::poll) else {
            self.task_suspension = Some(TaskSuspension::HostInfo(Box::new(wait)));
            return Ok(false);
        };
        let value = self.host_info_result(wait.operation, outcome, wait.location)?;
        self.store_intrinsic_result(
            Intrinsic::SystemInfo(wait.operation),
            wait.destination,
            Some(value),
        )?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abandoned_collection_keeps_admission_until_native_retirement_finishes() {
        let state = Arc::new(HostedState::new(crate::stdlib::Console::new(), vec![]));
        let job = Job::start(reserve(state.clone()).unwrap(), Op::Memory).unwrap();
        native::defer_joins(|| drop(job));
        assert!(state.host_info.busy.load(Ordering::Acquire));
        assert!(reserve(state.clone()).is_none());
        native::join_retired(&state);
        assert!(!state.host_info.busy.load(Ordering::Acquire));
        assert!(state.host_info.query(Op::Memory).is_ok());
    }
}
