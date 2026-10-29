//! VM-owned interactive processes with serialized, nonblocking binary pipe probes.

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::process::{ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, TryLockError};

use super::failure::{Failure, Kind};
use super::options::Request;
use super::platform::{GroupChild, PipeReader, PipeWriter};

const TAG: u64 = 0x5052_0000_0000_0000;
const MASK: u64 = 0xffff_0000_0000_0000;
const LIMIT: usize = 64;
pub(super) const MAX_BYTES: usize = 1_048_576;
static NEXT: AtomicU64 = AtomicU64::new(TAG | 1);

pub(super) struct Process {
    child: GroupChild,
    stdin: Option<PipeWriter>,
    stdout: Option<PipeReader>,
    stderr: Option<PipeReader>,
}

impl Process {
    pub(super) fn read(&mut self, maximum: i64, stderr: bool) -> Result<Option<Vec<u8>>, Failure> {
        let maximum = usize::try_from(maximum)
            .ok()
            .filter(|n| (1..=MAX_BYTES).contains(n))
            .ok_or_else(|| Failure::argument("MaxBytes", "Expected 1..1048576"))?;
        let pipe = if stderr {
            &mut self.stderr
        } else {
            &mut self.stdout
        };
        let Some(reader) = pipe else {
            return Ok(Some(Vec::new()));
        };
        let mut bytes = vec![0; maximum];
        match reader.read(&mut bytes) {
            Ok(count) => {
                bytes.truncate(count);
                if count == 0 {
                    *pipe = None;
                }
                Ok(Some(bytes))
            }
            Err(error) if pending(&error) => Ok(None),
            Err(error) => Err(Failure::new(Kind::Execution(error))),
        }
    }

    pub(super) fn write(&mut self, bytes: &[u8]) -> Result<usize, Failure> {
        let writer = self
            .stdin
            .as_mut()
            .ok_or_else(|| Failure::argument("Process", "Process stdin is closed"))?;
        match writer.write(bytes) {
            Ok(0) if !bytes.is_empty() => Err(Failure::new(Kind::Execution(
                io::ErrorKind::WriteZero.into(),
            ))),
            Ok(count) => Ok(count),
            Err(error) if pending(&error) => Ok(0),
            Err(error) => Err(Failure::new(Kind::Execution(error))),
        }
    }

    pub(super) fn close_stdin(&mut self) -> Result<bool, Failure> {
        let Some(writer) = &mut self.stdin else {
            return Ok(true);
        };
        match writer.flush() {
            Ok(()) => {
                self.stdin = None;
                Ok(true)
            }
            Err(error) if pending(&error) => Ok(false),
            Err(error) => Err(Failure::new(Kind::Execution(error))),
        }
    }

    pub(super) fn flush(&mut self) -> Result<bool, Failure> {
        let Some(writer) = &mut self.stdin else {
            return Ok(true);
        };
        match writer.flush() {
            Ok(()) => Ok(true),
            Err(error) if pending(&error) => Ok(false),
            Err(error) => Err(Failure::new(Kind::Execution(error))),
        }
    }

    pub(super) fn try_wait(&mut self) -> Result<Option<ExitStatus>, Failure> {
        self.child
            .try_wait()
            .map_err(|error| Failure::new(Kind::Wait(error)))
    }

    pub(super) fn terminate(&mut self) -> Result<(), Failure> {
        self.child
            .terminate_group()
            .map_err(|error| Failure::new(Kind::Execution(error)))
    }

    fn close(&mut self) -> Result<(), Failure> {
        self.terminate()?;
        self.stdin = None;
        self.child
            .wait()
            .map_err(|error| Failure::new(Kind::Wait(error)))?;
        // Drop pipes only after terminating their writer, including Windows I/O workers.
        self.stdout = None;
        self.stderr = None;
        Ok(())
    }
}

type Entry = Arc<Mutex<Option<Process>>>;

#[derive(Default)]
struct Registry {
    entries: HashMap<u64, Entry>,
    shutdown: bool,
}

#[derive(Default)]
pub(in crate::execution::vm::hosted) struct PipedProcesses {
    registry: Mutex<Registry>,
}

impl PipedProcesses {
    pub(super) fn start(&self, request: &Request<'_>) -> Result<u64, Failure> {
        request.validate()?;
        let mut registry = self.registry.lock().unwrap_or_else(|e| e.into_inner());
        if registry.shutdown {
            return Err(Failure::new(Kind::Cancelled));
        }
        if registry.entries.len() >= LIMIT {
            return Err(Failure::limit("interactive processes", LIMIT));
        }
        let handle = NEXT
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1).filter(|next| next & MASK == TAG)
            })
            .map_err(|_| Failure::limit("process handles", LIMIT))?;
        let mut command = request.command();
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child =
            GroupChild::spawn(command).map_err(|error| Failure::new(Kind::Start(error)))?;
        let stdin = child
            .take_stdin()
            .map_err(|error| Failure::new(Kind::Execution(error)))?;
        let stdout = child
            .take_stdout()
            .map_err(|error| Failure::new(Kind::Execution(error)))?;
        let stderr = child
            .take_stderr()
            .map_err(|error| Failure::new(Kind::Execution(error)))?;
        registry.entries.insert(
            handle,
            Arc::new(Mutex::new(Some(Process {
                child,
                stdin,
                stdout,
                stderr,
            }))),
        );
        Ok(handle)
    }

    pub(super) fn with<T>(
        &self,
        handle: u64,
        operation: impl FnOnce(&mut Process) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        let entry = self.entry(handle)?;
        let mut state = entry.lock().unwrap_or_else(|e| e.into_inner());
        operation(state.as_mut().ok_or_else(closed)?)
    }

    /// A busy alias must not hold up the selecting VM worker.
    pub(super) fn try_with<T>(
        &self,
        handle: u64,
        operation: impl FnOnce(&mut Process) -> Result<Option<T>, Failure>,
    ) -> Result<Option<T>, Failure> {
        let entry = {
            let registry = match self.registry.try_lock() {
                Ok(registry) => registry,
                Err(TryLockError::WouldBlock) => return Ok(None),
                Err(TryLockError::Poisoned(error)) => error.into_inner(),
            };
            registry.entries.get(&handle).cloned().ok_or_else(closed)?
        };
        let mut state = match entry.try_lock() {
            Ok(state) => state,
            Err(TryLockError::WouldBlock) => return Ok(None),
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
        };
        operation(state.as_mut().ok_or_else(closed)?)
    }

    fn entry(&self, handle: u64) -> Result<Entry, Failure> {
        self.registry
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entries
            .get(&handle)
            .cloned()
            .ok_or_else(closed)
    }

    pub(super) fn close(&self, handle: u64) -> Result<(), Failure> {
        let entry = self.entry(handle)?;
        let mut state = entry.lock().unwrap_or_else(|e| e.into_inner());
        state.as_mut().ok_or_else(closed)?.close()?;
        let process = state.take();
        drop(state);
        self.registry
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entries
            .remove(&handle);
        drop(process);
        Ok(())
    }

    pub(in crate::execution::vm::hosted) fn shutdown(&self) {
        let mut registry = self.registry.lock().unwrap_or_else(|e| e.into_inner());
        registry.shutdown = true;
        for entry in registry.entries.values() {
            if let Some(process) = entry.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
                let _ = process.terminate();
            }
        }
    }

    pub(in crate::execution::vm::hosted) fn wait_idle(&self) {
        let entries = std::mem::take(
            &mut self
                .registry
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .entries,
        );
        for entry in entries.into_values() {
            if let Some(mut process) = entry.lock().unwrap_or_else(|e| e.into_inner()).take() {
                let _ = process.close();
            }
        }
    }
}

impl Drop for PipedProcesses {
    fn drop(&mut self) {
        self.shutdown();
        self.wait_idle();
    }
}

fn closed() -> Failure {
    Failure::argument(
        "Process",
        "Process handle is closed or belongs to another VM",
    )
}
fn pending(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
    )
}
