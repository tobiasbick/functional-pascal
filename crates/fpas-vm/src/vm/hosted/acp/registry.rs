//! VM-local connection/endpoint identities, with launch reservations before native effects.

use super::super::proc::{AcpLaunch, AcpProcess, AcpTerminals, PipedProcesses};
use fpas_acp::{
    connection::{Connection, Fault, Limits, Responder, Runtime, Settings},
    transport::MessageEndpoint,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct State {
    runtime: Option<Arc<Runtime>>,
    connections: BTreeMap<u64, Connection>,
    endpoints: BTreeMap<u64, (u64, MessageEndpoint)>,
    terminals: BTreeMap<u64, Arc<AcpTerminals>>,
    opening: usize,
    stopping: bool,
}

#[derive(Default)]
pub(in crate::vm) struct Registry {
    state: Mutex<State>,
}

struct Reservation<'a>(&'a Registry);
impl Drop for Reservation<'_> {
    fn drop(&mut self) {
        self.0
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .opening -= 1;
    }
}

impl Registry {
    fn reserve(&self, limits: &Limits) -> Result<(Reservation<'_>, Arc<Runtime>), Fault> {
        limits.validate()?;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.stopping {
            return Err(Fault::Closed);
        }
        if state.connections.len() + state.opening >= 16 {
            return Err(Fault::LimitExceeded("Connections", 16));
        }
        if state.runtime.is_none() {
            state.runtime = Some(Arc::new(Runtime::new()?));
        }
        let runtime = state.runtime.as_ref().ok_or(Fault::Host)?.clone();
        state.opening += 1;
        Ok((Reservation(self), runtime))
    }

    pub(super) fn connect_messages(
        &self,
        settings: Settings,
        limits: Limits,
        processes: Arc<PipedProcesses>,
    ) -> Result<(u64, u64), Fault> {
        let (_reservation, runtime) = self.reserve(&limits)?;
        let (endpoint, transport) = fpas_acp::transport::messages(&limits)?;
        let terminals = AcpTerminals::new(processes, limits.clone());
        let connection = runtime.connect_configured(
            settings,
            limits,
            Arc::new(super::host::resources::OwnedTransport {
                inner: Arc::new(transport),
                terminals: terminals.clone(),
            }),
        )?;
        let id = connection.id();
        let endpoint_id = (id & 0x0000_ffff_ffff_ffff) | 0x4145_0000_0000_0000;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.stopping {
            drop(state);
            let _ = connection.close();
            return Err(Fault::Closed);
        }
        state.connections.insert(id, connection);
        state.terminals.insert(id, terminals);
        state.endpoints.insert(endpoint_id, (id, endpoint));
        Ok((id, endpoint_id))
    }

    pub(super) fn connect_process(
        &self,
        settings: Settings,
        limits: Limits,
        launch: AcpLaunch,
        processes: Arc<PipedProcesses>,
    ) -> Result<u64, Fault> {
        let (_reservation, runtime) = self.reserve(&limits)?;
        settings.requested_version()?;
        let terminals = AcpTerminals::new(processes.clone(), limits.clone());
        let transport = AcpProcess::start(processes, &launch, limits.clone())?;
        let connection = runtime.connect_configured(
            settings,
            limits,
            Arc::new(super::host::resources::OwnedTransport {
                inner: transport,
                terminals: terminals.clone(),
            }),
        )?;
        let id = connection.id();
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.stopping {
            drop(state);
            let _ = connection.close();
            return Err(Fault::Closed);
        }
        state.connections.insert(id, connection);
        state.terminals.insert(id, terminals);
        Ok(id)
    }

    pub(super) fn connection(&self, id: u64) -> Result<Connection, Fault> {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .connections
            .get(&id)
            .cloned()
            .ok_or(Fault::Closed)
    }

    pub(super) fn endpoint(&self, id: u64) -> Result<MessageEndpoint, Fault> {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .endpoints
            .get(&id)
            .map(|(_, endpoint)| endpoint.clone())
            .ok_or(Fault::Closed)
    }

    pub(super) fn responder(&self, id: u64) -> Result<(Connection, Responder), Fault> {
        let connections: Vec<_> = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .connections
            .values()
            .cloned()
            .collect();
        for connection in connections {
            if let Ok(responder) = connection.responder(id) {
                return Ok((connection, responder));
            }
        }
        Err(Fault::Closed)
    }

    pub(super) fn release(&self, connection: &Connection) -> Result<bool, Fault> {
        let result = connection.release()?;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.connections.remove(&connection.id());
        state.terminals.remove(&connection.id());
        state
            .endpoints
            .retain(|_, (owner, _)| *owner != connection.id());
        Ok(result)
    }
    pub(super) fn mcp_connection(&self, id: u64) -> Result<Connection, Fault> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .connections
            .values()
            .find(|connection| connection.mcp_endpoint(id).is_ok())
            .cloned()
            .ok_or(Fault::Closed)
    }
    pub(super) fn terminals(&self, connection: u64) -> Result<Arc<AcpTerminals>, Fault> {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .terminals
            .get(&connection)
            .cloned()
            .ok_or(Fault::Closed)
    }

    pub(in crate::vm) fn shutdown(&self) {
        let runtime = {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.stopping = true;
            state.runtime.clone()
        };
        if let Some(runtime) = runtime {
            runtime.request_shutdown();
        }
    }

    pub(in crate::vm) fn wait_idle(&self) {
        self.shutdown();
        let runtime = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .runtime
            .clone();
        if let Some(runtime) = runtime {
            runtime.shutdown();
        }
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.connections.clear();
        state.terminals.clear();
        state.endpoints.clear();
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        self.wait_idle();
    }
}
