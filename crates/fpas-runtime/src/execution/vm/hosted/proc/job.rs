//! Bounded native execution includes startup, host receivers, foreground restore and reaping.

use super::{
    engine,
    failure::{Failure, Kind},
    owned::OwnedRequest,
};
use crate::execution::vm::hosted::HostedState;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::JoinHandle,
};

pub(super) const LIMIT: usize = 64;

pub(super) enum Operation {
    Run(OwnedRequest),
    Interactive(OwnedRequest),
    Start(OwnedRequest),
    Close(u64),
}

pub(super) enum Output {
    Run(engine::Output),
    Interactive(std::process::ExitStatus),
    Started(u64),
    Closed,
}

struct Permit(Arc<HostedState>);

impl Drop for Permit {
    fn drop(&mut self) {
        self.0.process_jobs.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(super) struct Job {
    permit: Option<Permit>,
    stop: Arc<AtomicBool>,
    receiver: Mutex<mpsc::Receiver<Result<Output, Failure>>>,
    thread: Option<JoinHandle<()>>,
}

impl Job {
    pub(super) fn start(state: Arc<HostedState>, operation: Operation) -> Result<Self, Failure> {
        state
            .process_jobs
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < LIMIT).then_some(count + 1)
            })
            .map_err(|_| Failure::limit("native process jobs", LIMIT))?;
        let permit = Permit(state.clone());
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let (send, receiver) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("fpas-process".into())
            .spawn(move || {
                super::super::native::defer_joins(|| {
                    let cancelled = |token: Option<u64>| {
                        stopped.load(Ordering::Acquire)
                            || token.is_some_and(|token| {
                                state.cancellations.is_cancelled(token).unwrap_or(true)
                            })
                    };
                    let result = match operation {
                        Operation::Run(request) => {
                            let receiver = if request.capture {
                                None
                            } else {
                                state
                                    .program_stderr
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner())
                                    .clone()
                            };
                            state
                                .processes
                                .run_at(
                                    &request.borrow(),
                                    receiver,
                                    || cancelled(request.token),
                                    request.started,
                                )
                                .map(Output::Run)
                        }
                        Operation::Interactive(request) => {
                            let terminal = state
                                .process_terminal
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .clone();
                            state
                                .processes
                                .run_interactive_at(
                                    &request.borrow(),
                                    terminal.as_ref(),
                                    || cancelled(request.token),
                                    request.started,
                                )
                                .map(Output::Interactive)
                        }
                        Operation::Start(request) => state
                            .piped_processes
                            .start_with_cancellation(&request.borrow(), || cancelled(request.token))
                            .map(Output::Started),
                        Operation::Close(handle) => {
                            state.piped_processes.close(handle).map(|()| Output::Closed)
                        }
                    };
                    let _ = send.send(result);
                })
            })
            .map_err(|error| Failure::new(Kind::Execution(error)))?;
        Ok(Self {
            permit: Some(permit),
            stop,
            receiver: Mutex::new(receiver),
            thread: Some(thread),
        })
    }

    pub(super) fn poll(&self) -> Option<Result<Output, Failure>> {
        match self
            .receiver
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .try_recv()
        {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(Failure::new(Kind::Execution(
                std::io::Error::other("Native process worker exited without a result"),
            )))),
        }
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let Some(permit) = self.permit.take() else {
            return;
        };
        let (_, empty) = mpsc::channel();
        let receiver = std::mem::replace(
            self.receiver.get_mut().unwrap_or_else(|e| e.into_inner()),
            empty,
        );
        let retired = Retired {
            permit,
            receiver,
            thread: self.thread.take(),
        };
        let state = retired.permit.0.clone();
        super::super::native::retire(&state, move || retired.finish());
    }
}

pub(in crate::execution::vm::hosted) struct Retired {
    permit: Permit,
    receiver: mpsc::Receiver<Result<Output, Failure>>,
    thread: Option<JoinHandle<()>>,
}

impl Retired {
    fn finish(mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        if let Ok(Ok(Output::Started(handle))) = self.receiver.try_recv() {
            let _ = self.permit.0.piped_processes.close(handle);
        }
    }
}
