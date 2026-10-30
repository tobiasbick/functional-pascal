//! Bounded native entropy jobs; queued calls and abandoned results remain owned.

use super::super::{HostedState, crypto, native, random};
use super::Source;
use crate::execution::vm::{VmError, tasks::TaskSuspension, worker::Worker};
use crate::{Intrinsic, Register, SourceLocation, Value};
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

#[derive(Clone, Copy)]
enum Request {
    Crypto(crypto::Request),
    Seed,
}
enum Output {
    Crypto(crypto::Output),
    Seed([u8; 32]),
}
#[derive(Clone, Copy)]
enum Operation {
    Crypto(crate::CryptoIntrinsic),
    Random(random::Call),
}
enum State {
    Queued(Request),
    Running(Job),
}
pub(in crate::execution::vm) struct Wait {
    state: State,
    operation: Operation,
    intrinsic: Intrinsic,
    destination: Option<Register>,
    location: SourceLocation,
}
struct Permit(Arc<HostedState>);
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.entropy_jobs.fetch_sub(1, Ordering::AcqRel);
    }
}
fn reserve(state: Arc<HostedState>) -> Option<Permit> {
    state
        .entropy_jobs
        .try_update(Ordering::AcqRel, Ordering::Acquire, |n| {
            (n < LIMIT).then_some(n + 1)
        })
        .ok()
        .map(|_| Permit(state))
}
type Outcome = Option<Result<Output, getrandom::Error>>;
struct Job {
    permit: Option<Permit>,
    stop: Arc<AtomicBool>,
    receiver: Option<Mutex<mpsc::Receiver<Outcome>>>,
    thread: Option<JoinHandle<()>>,
}
fn host_error(location: SourceLocation, error: impl std::fmt::Display) -> VmError {
    crate::stdlib::error::std_runtime_error(
        fpas_diagnostics::codes::RUNTIME_HOST_IO_FAILURE,
        format!("Native entropy worker failed: {error}"),
        "Retry after earlier native work has finished.",
        location,
    )
}
impl Job {
    fn start(permit: Permit, request: Request, location: SourceLocation) -> Result<Self, VmError> {
        let source: Arc<Source> = permit.0.entropy_source.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let (send, receiver) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("fpas-entropy".into())
            .spawn(move || {
                native::defer_joins(|| {
                    // Check between rejection samples as well as around a complete OS call.
                    let mut cancelled = false;
                    let mut fill = |bytes: &mut [u8]| {
                        if stopped.load(Ordering::Acquire) {
                            cancelled = true;
                            return Err(getrandom::Error::UNSUPPORTED);
                        }
                        source.fill(bytes)
                    };
                    let result = match request {
                        Request::Crypto(request) => request.run(&mut fill).map(Output::Crypto),
                        Request::Seed => {
                            let mut seed = [0; 32];
                            fill(&mut seed).map(|()| Output::Seed(seed))
                        }
                    };
                    let outcome =
                        (!cancelled && !stopped.load(Ordering::Acquire)).then_some(result);
                    let _ = send.send(outcome);
                });
            })
            .map_err(|e| host_error(location, e))?;
        Ok(Self {
            permit: Some(permit),
            stop,
            receiver: Some(Mutex::new(receiver)),
            thread: Some(thread),
        })
    }
    fn poll(&self, location: SourceLocation) -> Result<Option<Outcome>, VmError> {
        let receiver = self
            .receiver
            .as_ref()
            .unwrap_or_else(|| unreachable!("live entropy job"));
        match receiver
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .try_recv()
        {
            Ok(value) => Ok(Some(value)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(host_error(location, "worker exited without a result"))
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
    pub(in crate::execution::vm) fn start_entropy(
        &mut self,
        intrinsic: Intrinsic,
        values: &[Value],
        destination: Option<Register>,
    ) -> Result<(), VmError> {
        let location = self.intrinsic_location();
        let (operation, request) = match intrinsic {
            Intrinsic::Crypto(op) => match crypto::Request::prepare(self, op, values)? {
                Ok(request) => (Operation::Crypto(op), Request::Crypto(request)),
                Err(error) => {
                    let value = self.crypto_result(op, Err(error), location)?;
                    return self.store_intrinsic_result(intrinsic, destination, Some(value));
                }
            },
            Intrinsic::Random(op) => {
                let call = random::Call::prepare(self, op, values)?;
                let value = {
                    let mut state = self.hosted.random.lock().unwrap_or_else(|p| p.into_inner());
                    call.ready(&mut state)
                };
                if let Some(value) = value {
                    return self.store_intrinsic_result(intrinsic, destination, Some(value));
                }
                (Operation::Random(call), Request::Seed)
            }
            _ => unreachable!("entropy intrinsic"),
        };
        let state = match reserve(self.hosted.clone()) {
            Some(permit) => State::Running(Job::start(permit, request, location)?),
            None => State::Queued(request),
        };
        self.task_suspension = Some(TaskSuspension::Entropy(Box::new(Wait {
            state,
            operation,
            intrinsic,
            destination,
            location,
        })));
        self.suspend_requested = true;
        let _ = self.poll_task_suspension()?;
        Ok(())
    }
    pub(in crate::execution::vm) fn poll_entropy(
        &mut self,
        mut wait: Wait,
    ) -> Result<bool, VmError> {
        if let State::Queued(request) = wait.state {
            let Some(permit) = reserve(self.hosted.clone()) else {
                self.task_suspension = Some(TaskSuspension::Entropy(Box::new(wait)));
                return Ok(false);
            };
            wait.state = State::Running(Job::start(permit, request, wait.location)?);
        }
        let State::Running(job) = &wait.state else {
            unreachable!("admitted entropy job");
        };
        let Some(outcome) = job.poll(wait.location)? else {
            self.task_suspension = Some(TaskSuspension::Entropy(Box::new(wait)));
            return Ok(false);
        };
        let outcome = outcome.ok_or_else(|| {
            self.runtime_error(
                fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED,
                "Entropy operation was cancelled",
                "Retry from an active task.",
            )
        })?;
        let value = match wait.operation {
            Operation::Crypto(op) => {
                let result = outcome
                    .map(|out| match out {
                        Output::Crypto(v) => v,
                        _ => unreachable!("crypto result"),
                    })
                    .map_err(crypto::CryptoFailure::Entropy);
                self.crypto_result(op, result, wait.location)?
            }
            Operation::Random(call) => {
                let seed = outcome.map(|out| match out {
                    Output::Seed(seed) => seed,
                    _ => unreachable!("seed result"),
                });
                let mut state = self.hosted.random.lock().unwrap_or_else(|p| p.into_inner());
                call.finish(&mut state, seed)
                    .map_err(|e| self.random_source_error(e))?
            }
        };
        self.store_intrinsic_result(wait.intrinsic, wait.destination, Some(value))?;
        Ok(true)
    }
}
