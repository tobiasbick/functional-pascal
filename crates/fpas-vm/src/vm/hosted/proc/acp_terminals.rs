//! ACP terminal leases use the existing owned process registry and pipe probes.

use super::{
    acp::{AcpLaunch, fault},
    acp_output::{Decoder, Output},
    interactive::PipedProcesses,
    options::{Options, Request},
};
use fpas_acp::{
    connection::{Fault, Limits},
    protocol::v1 as sdk,
};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

static NEXT: AtomicU64 = AtomicU64::new(1);
struct Entry {
    session: String,
    handle: Option<u64>,
    closed: bool,
    maximum: usize,
    decoders: [Decoder; 2],
    eof: [bool; 2],
    output: Output,
    status: Option<sdk::TerminalExitStatus>,
}
#[derive(Default)]
struct State {
    entries: BTreeMap<String, Arc<Mutex<Entry>>>,
    bytes: usize,
    closing: bool,
}
pub(in crate::vm::hosted) struct Terminals {
    processes: Arc<PipedProcesses>,
    limits: Limits,
    state: Mutex<State>,
    pub changed: tokio::sync::Notify,
}
pub(in crate::vm::hosted) struct TerminalReservation {
    pub id: String,
    entry: Arc<Mutex<Entry>>,
    owner: Arc<Terminals>,
    published: bool,
}
impl Drop for TerminalReservation {
    fn drop(&mut self) {
        if !self.published {
            let _ = self.owner.release(&self.id, None);
        }
    }
}

fn request<'a>(launch: &'a AcpLaunch) -> Request<'a> {
    Request {
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
    }
}
impl Terminals {
    pub fn new(processes: Arc<PipedProcesses>, limits: Limits) -> Arc<Self> {
        Arc::new(Self {
            processes,
            limits,
            state: Mutex::new(State::default()),
            changed: tokio::sync::Notify::new(),
        })
    }
    pub fn active(&self) -> bool {
        !self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entries
            .is_empty()
    }
    pub fn validate(&self, launch: &AcpLaunch) -> Result<(), Fault> {
        request(launch).validate().map_err(fault)
    }
    pub fn reserve(
        self: &Arc<Self>,
        session: &str,
        requested: Option<u64>,
    ) -> Result<TerminalReservation, Fault> {
        let maximum = requested
            .map_or(Ok(self.limits.terminal_bytes), usize::try_from)
            .map_err(|_| Fault::InvalidArgument("OutputByteLimit"))?;
        if maximum > self.limits.terminal_bytes {
            return Err(Fault::LimitExceeded(
                "TerminalBytes",
                self.limits.terminal_bytes,
            ));
        }
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.closing {
            return Err(Fault::Closed);
        }
        if state.entries.len() >= self.limits.terminals {
            return Err(Fault::LimitExceeded("Terminals", self.limits.terminals));
        }
        if maximum > self.limits.all_terminal_bytes.saturating_sub(state.bytes) {
            return Err(Fault::LimitExceeded(
                "AllTerminalBytes",
                self.limits.all_terminal_bytes,
            ));
        }
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let id = format!("fpas-terminal-{serial}");
        let entry = Arc::new(Mutex::new(Entry {
            session: session.into(),
            handle: None,
            closed: false,
            maximum,
            decoders: Default::default(),
            eof: [false; 2],
            output: Output::new(maximum),
            status: None,
        }));
        state.bytes += maximum;
        state.entries.insert(id.clone(), entry.clone());
        self.changed.notify_one();
        Ok(TerminalReservation {
            id,
            entry,
            owner: self.clone(),
            published: false,
        })
    }
    pub fn start(
        &self,
        reservation: &mut TerminalReservation,
        launch: &AcpLaunch,
    ) -> Result<(), Fault> {
        let mut entry = reservation.entry.lock().unwrap_or_else(|e| e.into_inner());
        if entry.closed {
            return Err(Fault::Closed);
        }
        let handle = self.processes.start(&request(launch)).map_err(fault)?;
        entry.handle = Some(handle);
        reservation.published = true;
        Ok(())
    }
    fn entry(&self, id: &str, session: Option<&str>) -> Result<Arc<Mutex<Entry>>, Fault> {
        let entry = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entries
            .get(id)
            .cloned()
            .ok_or(Fault::Closed)?;
        if session.is_some_and(|session| {
            entry.lock().unwrap_or_else(|e| e.into_inner()).session != session
        }) {
            return Err(Fault::InvalidArgument("TerminalSession"));
        }
        Ok(entry)
    }
    pub fn validate_id(&self, id: &str, session: &str) -> Result<(), Fault> {
        self.entry(id, Some(session)).map(|_| ())
    }
    fn drain(&self, entry: &mut Entry) -> Result<(), Fault> {
        if entry.closed {
            return Err(Fault::Closed);
        }
        let Some(handle) = entry.handle else {
            return Ok(());
        };
        for _ in 0..4 {
            for stream in 0..2 {
                if let Some(bytes) = self
                    .processes
                    .with(handle, |process| process.read(4096, stream == 1))
                    .map_err(fault)?
                {
                    let text = entry.decoders[stream].push(&bytes, bytes.is_empty());
                    entry.eof[stream] |= bytes.is_empty();
                    entry.output.push(&text);
                }
            }
        }
        if let Some(status) = self
            .processes
            .with(handle, |process| process.try_wait())
            .map_err(fault)?
        {
            entry.status = Some(status_value(status));
        }
        Ok(())
    }
    pub fn pump(&self) -> Result<(), Fault> {
        let entries: Vec<_> = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entries
            .values()
            .cloned()
            .collect();
        for entry in entries {
            let mut entry = entry.lock().unwrap_or_else(|e| e.into_inner());
            if !entry.closed {
                self.drain(&mut entry)?;
            }
        }
        Ok(())
    }
    pub fn output(&self, id: &str, session: &str) -> Result<sdk::TerminalOutputResponse, Fault> {
        let entry = self.entry(id, Some(session))?;
        let mut entry = entry.lock().unwrap_or_else(|e| e.into_inner());
        self.drain(&mut entry)?;
        Ok(
            sdk::TerminalOutputResponse::new(entry.output.text.clone(), entry.output.truncated)
                .exit_status(entry.status.clone()),
        )
    }
    pub fn status(
        &self,
        id: &str,
        session: &str,
    ) -> Result<Option<sdk::TerminalExitStatus>, Fault> {
        let entry = self.entry(id, Some(session))?;
        let mut entry = entry.lock().unwrap_or_else(|e| e.into_inner());
        self.drain(&mut entry)?;
        Ok(if entry.eof.iter().all(|eof| *eof) {
            entry.status.clone()
        } else {
            None
        })
    }
    pub fn kill(&self, id: &str, session: &str) -> Result<(), Fault> {
        let entry = self.entry(id, Some(session))?;
        let mut entry = entry.lock().unwrap_or_else(|e| e.into_inner());
        self.drain(&mut entry)?;
        if let Some(handle) = entry.handle {
            self.processes
                .with(handle, |process| process.terminate())
                .map_err(fault)?;
        }
        Ok(())
    }
    pub fn release(&self, id: &str, session: Option<&str>) -> Result<(), Fault> {
        let entry = self.entry(id, session)?;
        let mut owned = entry.lock().unwrap_or_else(|e| e.into_inner());
        if owned.closed {
            return Err(Fault::Closed);
        }
        if let Some(handle) = owned.handle {
            self.processes.close(handle).map_err(fault)?;
        }
        owned.closed = true;
        let maximum = owned.maximum;
        drop(owned);
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.entries.remove(id).is_some() {
            state.bytes -= maximum;
        }
        Ok(())
    }
    pub fn close(&self, session: Option<&str>) -> Result<(), Fault> {
        let entries = {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if session.is_none() {
                state.closing = true;
            }
            state
                .entries
                .iter()
                .filter(|(_, entry)| {
                    session.is_none_or(|session| {
                        entry.lock().unwrap_or_else(|e| e.into_inner()).session == session
                    })
                })
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>()
        };
        let mut failed = false;
        for id in entries {
            if self.release(&id, None).is_err() {
                failed = true;
            }
        }
        if failed { Err(Fault::Host) } else { Ok(()) }
    }
}
fn status_value(status: std::process::ExitStatus) -> sdk::TerminalExitStatus {
    let value = sdk::TerminalExitStatus::new().exit_code(status.code().map(|code| code as u32));
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        return value.signal(status.signal().map(|signal| signal.to_string()));
    }
    #[cfg(not(unix))]
    {
        value
    }
}
