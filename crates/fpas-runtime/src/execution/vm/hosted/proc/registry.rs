//! Admission and process-group ownership without waiting under the polling mutex.

use std::collections::HashMap;
use std::io;
use std::process::{Command, ExitStatus};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use super::failure::{Failure, Kind};
use super::platform::GroupChild;

pub(super) struct Process {
    pub child: Mutex<Option<GroupChild>>,
    stopped: AtomicBool,
    done: Mutex<bool>,
    completed: Condvar,
}

impl Process {
    pub(super) fn id(&self) -> io::Result<u32> {
        self.child
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .map(GroupChild::id)
            .ok_or_else(|| io::Error::other("Process ownership lost"))
    }

    pub(super) fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }

    pub(super) fn spawn(&self, command: Command) -> io::Result<()> {
        let child = GroupChild::spawn(command)?;
        *self.child.lock().unwrap_or_else(|error| error.into_inner()) = Some(child);
        if self.stopped() {
            self.terminate()?;
        }
        Ok(())
    }

    pub(super) fn poll(&self) -> io::Result<Option<ExitStatus>> {
        self.child
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_mut()
            .map_or(Ok(None), GroupChild::try_wait)
    }

    pub(super) fn terminate(&self) -> io::Result<()> {
        self.child
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_mut()
            .map_or(Ok(()), GroupChild::terminate_group)
    }

    pub(super) fn reap(&self) -> io::Result<Option<ExitStatus>> {
        // Transfer ownership before waiting so shutdown never blocks on this mutex.
        let child = self
            .child
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        child.map(|mut child| child.wait()).transpose()
    }
}

struct State {
    entries: HashMap<u64, Arc<Process>>,
    shutdown: bool,
    limit: usize,
}

pub(in crate::execution::vm::hosted) struct Processes {
    state: Arc<Mutex<State>>,
    next: AtomicU64,
}

impl Processes {
    pub(in crate::execution::vm::hosted) fn new() -> Self {
        Self::with_limit(64)
    }

    pub(super) fn with_limit(limit: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                entries: HashMap::new(),
                shutdown: false,
                limit,
            })),
            next: AtomicU64::new(1),
        }
    }

    pub(super) fn reserve(&self) -> Result<Reservation, Failure> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.shutdown {
            return Err(Failure::new(Kind::Cancelled));
        }
        if state.entries.len() >= state.limit {
            return Err(Failure::limit("processes", state.limit));
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let process = Arc::new(Process {
            child: Mutex::new(None),
            stopped: AtomicBool::new(false),
            done: Mutex::new(false),
            completed: Condvar::new(),
        });
        state.entries.insert(id, Arc::clone(&process));
        Ok(Reservation {
            registry: Arc::clone(&self.state),
            id,
            process,
        })
    }

    /// Only request cancellation here: a stderr receiver may reenter VM shutdown.
    pub(in crate::execution::vm::hosted) fn shutdown(&self) {
        let entries = {
            let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            state.shutdown = true;
            state.entries.values().cloned().collect::<Vec<_>>()
        };
        for process in entries {
            process.stopped.store(true, Ordering::Release);
            let _ = process.terminate();
        }
    }

    /// The VM calls this after its workers return, never inside an active receiver.
    pub(in crate::execution::vm::hosted) fn wait_idle(&self) {
        let entries = self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .entries
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for process in entries {
            let mut done = process
                .done
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            while !*done {
                done = process
                    .completed
                    .wait(done)
                    .unwrap_or_else(|error| error.into_inner());
            }
        }
    }
}

impl Drop for Processes {
    fn drop(&mut self) {
        self.shutdown();
        // Saved executions own their reservations and reap on drop. Waiting here
        // could block destruction of a worker that still owns such an execution.
        // The VM's explicit wait_idle barrier runs after its workers return.
    }
}

pub(super) struct Reservation {
    registry: Arc<Mutex<State>>,
    id: u64,
    pub process: Arc<Process>,
}

impl Drop for Reservation {
    fn drop(&mut self) {
        // Also runs on panic, including a panicking host stderr receiver.
        let _ = self.process.terminate();
        let _ = self.process.reap();
        self.registry
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .entries
            .remove(&self.id);
        *self
            .process
            .done
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = true;
        self.process.completed.notify_all();
    }
}
