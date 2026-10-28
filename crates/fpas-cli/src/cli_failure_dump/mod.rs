//! Opt-in failure reporting through immutable VM data and an independent writer thread.
//!
//! Contract: `docs/pascal/tools/failure-dump.md`.

mod budget;
mod destination;
mod plan;
mod publication;
mod sources;
use fpas_debug::snapshot_value as value;
mod wire;

use fpas_diagnostics::Diagnostic;
use fpas_runtime::execution::{FailureObservation, Vm};
use std::{
    io,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
    thread::JoinHandle,
};

pub(crate) use destination::validate as validate_destination;
pub(crate) use publication::publish as publish_report;
pub(crate) use sources::Sources;

enum Message {
    Observed(Arc<FailureObservation>),
    Fallback(Box<Diagnostic>, bool),
    Success,
}

/// One optional writer, joined only after ordinary VM cleanup has completed.
pub(crate) struct Session {
    sender: SyncSender<Message>,
    observed: Arc<AtomicBool>,
    writer: JoinHandle<io::Result<()>>,
}

impl Session {
    /// Start the independent writer and install the immutable handoff before execution.
    pub(crate) fn start(vm: &mut Vm, path: PathBuf, sources: Sources) -> io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let observed = Arc::new(AtomicBool::new(false));
        let writer = std::thread::Builder::new()
            .name("fpas-failure-dump".into())
            .spawn(move || {
                let bytes = match receiver.recv() {
                    Ok(Message::Observed(data)) => {
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            plan::observation(&data, &sources)
                        }))
                        .unwrap_or_else(|_| {
                            plan::minimal(
                                &data.error,
                                data.cause.task_id,
                                &sources,
                                data.error.reason,
                            )
                        })?
                    }
                    Ok(Message::Fallback(error, clipped)) => {
                        plan::minimal(&error, None, &sources, clipped.then_some("byte_limit"))?
                    }
                    Ok(Message::Success) | Err(_) => return Ok(()),
                };
                publication::publish(&path, &bytes)
            })?;
        let destination = sender.clone();
        let delivered = Arc::clone(&observed);
        if !vm.set_failure_observer(move |data| {
            if destination.send(Message::Observed(data)).is_ok() {
                delivered.store(true, Ordering::Release);
            }
        }) {
            let _ = sender.send(Message::Success);
            let _ = writer.join();
            return Err(io::Error::other(
                "Failure observer could not be installed before execution",
            ));
        }
        Ok(Self {
            sender,
            observed,
            writer,
        })
    }

    /// Finish without replacing the program diagnostic; fallback text is bounded before copying.
    pub(crate) fn finish(self, error: Option<&Diagnostic>) -> io::Result<()> {
        if !self.observed.load(Ordering::Acquire) {
            let message = match error {
                Some(error) => {
                    let mut copy = Box::new(Diagnostic::error_without_source(
                        error.code,
                        wire::prefix(&error.message),
                        error.help.as_deref().map(wire::prefix).map(str::to_owned),
                    ));
                    copy.span = error.span;
                    let clipped = copy.message.len() < error.message.len()
                        || copy.help.as_deref().map_or(0, str::len)
                            < error.help.as_deref().map_or(0, str::len);
                    Message::Fallback(copy, clipped)
                }
                None => Message::Success,
            };
            let _ = self.sender.send(message);
        }
        self.writer
            .join()
            .map_err(|_| io::Error::other("Failure report writer panicked"))?
    }
}
