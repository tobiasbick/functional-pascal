//! ACP framing over the existing single-owner process registry and pipe probes.

use super::{
    failure::{Failure, Kind},
    interactive::PipedProcesses,
    options::{Options, Request},
};
use fpas_acp::{
    RawJson,
    connection::{Fault, Limits},
    transport::{ExitStatus, Frame, Framer, Transport, TransportStatus},
};
use futures::future::BoxFuture;
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

#[cfg(test)]
mod tests;

/// Detached launch data, validated by the same preflight as Std.Proc.
#[derive(Clone)]
pub(in crate::execution::vm::hosted) struct AcpLaunch {
    pub command: String,
    pub arguments: Vec<String>,
    pub directory: Option<String>,
    pub environment: Vec<(String, Option<String>)>,
}

struct Input {
    framer: Framer,
    frames: VecDeque<RawJson>,
    eof: bool,
}

/// The registry owns the child; this adapter owns only its private registry lease.
pub(in crate::execution::vm::hosted) struct AcpProcess {
    processes: Arc<PipedProcesses>,
    handle: u64,
    limits: Limits,
    input: Mutex<Input>,
    stderr: Mutex<(Vec<u8>, bool)>,
    closed: AtomicBool,
}

impl AcpProcess {
    pub(in crate::execution::vm::hosted) fn start(
        processes: Arc<PipedProcesses>,
        launch: &AcpLaunch,
        limits: Limits,
    ) -> Result<Arc<Self>, Fault> {
        limits.validate()?;
        let request = Request {
            command: &launch.command,
            arguments: launch.arguments.iter().map(String::as_str).collect(),
            options: Options {
                directory: launch.directory.as_deref(),
                environment: launch
                    .environment
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_deref()))
                    .collect(),
                ..Options::default()
            },
            capture: true,
            inherit_input: false,
        };
        let handle = processes.start(&request).map_err(fault)?;
        Ok(Arc::new(Self {
            processes,
            handle,
            input: Mutex::new(Input {
                framer: Framer::new(limits.message_bytes),
                frames: VecDeque::new(),
                eof: false,
            }),
            stderr: Mutex::new((Vec::new(), false)),
            limits,
            closed: AtomicBool::new(false),
        }))
    }

    fn stderr(&self) -> Result<(), Fault> {
        // Bound each probe batch as well as retained output; sustained output yields.
        for _ in 0..16 {
            let Some(bytes) = self
                .processes
                .with(self.handle, |process| process.read(4096, true))
                .map_err(fault)?
            else {
                break;
            };
            if bytes.is_empty() {
                break;
            }
            let mut stderr = self.stderr.lock().unwrap_or_else(|e| e.into_inner());
            let available = self.limits.stderr_bytes.saturating_sub(stderr.0.len());
            let count = bytes.len().min(available);
            stderr.0.extend_from_slice(&bytes[..count]);
            stderr.1 |= count != bytes.len();
        }
        Ok(())
    }

    fn probe(&self) -> Result<Option<Option<Frame>>, Fault> {
        self.stderr()?;
        let mut input = self.input.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(frame) = input.frames.pop_front() {
            return Ok(Some(Some(Frame::new(frame))));
        }
        if let Some(fault) = input.framer.failure() {
            return Err(fault.clone());
        }
        if input.eof {
            return Ok(Some(None));
        }
        // Observing an exited leader also terminates descendants that inherited pipes.
        self.processes
            .with(self.handle, |process| process.try_wait())
            .map_err(fault)?;
        let Some(bytes) = self
            .processes
            .with(self.handle, |process| process.read(4096, false))
            .map_err(fault)?
        else {
            return Ok(None);
        };
        if bytes.is_empty() {
            input.framer.finish()?;
            input.eof = true;
            return Ok(Some(None));
        }
        let frames = input.framer.push(&bytes)?;
        let bytes: usize = frames.iter().map(|frame| frame.text().len()).sum();
        if frames.len() > self.limits.queue_entries || bytes > self.limits.queue_bytes {
            return Err(Fault::LimitExceeded("InputQueue", self.limits.queue_bytes));
        }
        input.frames.extend(frames);
        Ok(input
            .frames
            .pop_front()
            .map(|frame| Some(Frame::new(frame))))
    }

    async fn cleanup(&self, grace_millis: u64) -> Result<TransportStatus, Fault> {
        let deadline = tokio::time::Instant::now() + Duration::from_millis(grace_millis);
        let mut terminated = false;
        let status = loop {
            self.stderr()?;
            // Output after termination admission is discarded while pipes keep draining.
            let _ = self
                .processes
                .with(self.handle, |process| process.read(4096, false))
                .map_err(fault)?;
            self.processes
                .with(self.handle, |process| process.close_stdin())
                .map_err(fault)?;
            if let Some(status) = self
                .processes
                .with(self.handle, |process| process.try_wait())
                .map_err(fault)?
            {
                break status;
            }
            if !terminated && tokio::time::Instant::now() >= deadline {
                self.processes
                    .with(self.handle, |process| process.terminate())
                    .map_err(fault)?;
                terminated = true;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        };
        // Drain pending stderr completions after exit, including Windows blocking I/O.
        loop {
            match self
                .processes
                .with(self.handle, |process| process.read(4096, true))
                .map_err(fault)?
            {
                Some(bytes) if bytes.is_empty() => break,
                Some(bytes) => {
                    let mut stderr = self.stderr.lock().unwrap_or_else(|e| e.into_inner());
                    let count = bytes
                        .len()
                        .min(self.limits.stderr_bytes.saturating_sub(stderr.0.len()));
                    stderr.0.extend_from_slice(&bytes[..count]);
                    stderr.1 |= count != bytes.len();
                }
                None => tokio::time::sleep(Duration::from_millis(1)).await,
            }
        }
        let processes = self.processes.clone();
        let handle = self.handle;
        // Windows pipe runtimes must be dropped outside an async execution context.
        tokio::task::spawn_blocking(move || processes.close(handle))
            .await
            .map_err(|_| Fault::Host)?
            .map_err(fault)?;
        self.closed.store(true, Ordering::Release);
        let stderr = self.stderr.lock().unwrap_or_else(|e| e.into_inner());
        let mut text = String::from_utf8_lossy(&stderr.0).into_owned();
        let mut truncated = stderr.1 || text.len() > self.limits.stderr_bytes;
        if text.len() > self.limits.stderr_bytes {
            let mut boundary = self.limits.stderr_bytes;
            while !text.is_char_boundary(boundary) {
                boundary -= 1;
            }
            text.truncate(boundary);
            truncated = true;
        }
        Ok(TransportStatus {
            exit: Some(exit_status(status)),
            stderr: text,
            stderr_truncated: truncated,
        })
    }
}

impl Transport for AcpProcess {
    fn receive(&self) -> BoxFuture<'_, Result<Option<Frame>, Fault>> {
        Box::pin(async {
            loop {
                if let Some(frame) = self.probe()? {
                    return Ok(frame);
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
    }

    fn send(&self, frame: RawJson) -> BoxFuture<'_, Result<(), Fault>> {
        Box::pin(async move {
            if frame.text().len() > self.limits.message_bytes {
                return Err(Fault::LimitExceeded(
                    "MessageBytes",
                    self.limits.message_bytes,
                ));
            }
            let mut bytes = frame.text().as_bytes().to_vec();
            bytes.push(b'\n');
            let mut written = 0;
            while written < bytes.len() {
                self.stderr()?;
                let count = self
                    .processes
                    .with(self.handle, |process| process.write(&bytes[written..]))
                    .map_err(fault)?;
                written += count;
                if count == 0 {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
            // Tokio's Windows writer accepts a chunk before its blocking write finishes.
            loop {
                self.stderr()?;
                if self
                    .processes
                    .with(self.handle, |process| process.flush())
                    .map_err(fault)?
                {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
    }

    fn finish(&self, grace_millis: u64) -> BoxFuture<'_, Result<TransportStatus, Fault>> {
        Box::pin(self.cleanup(grace_millis))
    }
}

impl Drop for AcpProcess {
    fn drop(&mut self) {
        if !self.closed.load(Ordering::Acquire) {
            let processes = self.processes.clone();
            let handle = self.handle;
            // Closing also reaps the child and returns its process-registry slot.
            // Windows pipe runtimes must be dropped outside async execution.
            if let Ok(runtime) = tokio::runtime::Handle::try_current() {
                runtime.spawn_blocking(move || processes.close(handle));
            } else {
                let _ = processes.close(handle);
            }
        }
    }
}

pub(super) fn fault(error: Failure) -> Fault {
    match error.kind {
        Kind::Argument(name, _) => Fault::InvalidArgument(name),
        Kind::Limit(resource, maximum) => Fault::LimitExceeded(resource, maximum),
        Kind::Cancelled => Fault::Cancelled,
        Kind::Timeout => Fault::Timeout,
        _ => Fault::Host,
    }
}

fn exit_status(status: std::process::ExitStatus) -> ExitStatus {
    if let Some(code) = status.code() {
        return ExitStatus::Exited(code as u32);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return ExitStatus::Signaled(signal);
        }
    }
    ExitStatus::Unknown
}
