//! VM-owned callback roots; native transports retain only opaque registration keys.

use fpas_bytecode::{RootValues, Value};
use fpas_mcp::{
    Catalog, CleanupReceipt, Fault, HttpOptions, HttpRuntime, Publication, PublishTicket, Server,
    model::Tool,
};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicU64, Ordering},
    },
};

static NEXT: AtomicU64 = AtomicU64::new(1);
fn identity() -> Result<u64, Fault> {
    NEXT.try_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
        n.checked_add(1).filter(|v| *v <= i64::MAX as u64)
    })
    .map_err(|_| Fault::Closed)
}

struct Entry {
    catalog: Option<Catalog>,
    server: Option<Server>,
    handlers: BTreeMap<u64, RootValues>,
    next_handler: u64,
    closed: bool,
}
struct Published {
    server: u64,
    publication: Arc<Publication>,
}
#[derive(Default)]
struct State {
    stopping: bool,
    servers: BTreeMap<u64, Entry>,
    publications: BTreeMap<u64, Published>,
    claims: BTreeMap<u64, Weak<Claim>>,
    runtime: Option<Arc<HttpRuntime>>,
    owned: BTreeMap<u64, u64>,
}
#[derive(Default)]
pub(in crate::vm) struct Registry {
    state: Arc<Mutex<State>>,
}

pub(super) struct Claim {
    pub id: u64,
    pub server: u64,
    pub responder: fpas_mcp::Responder,
    state: Weak<Mutex<State>>,
}
impl Drop for Claim {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .claims
                .remove(&self.id);
        }
    }
}
impl Registry {
    pub(super) fn owned(&self, connection: u64) -> Option<u64> {
        self.lock().owned.get(&connection).copied()
    }
    pub(super) fn register_owned(
        &self,
        connection: u64,
        tool: Tool,
        handler: RootValues,
    ) -> Result<(), Fault> {
        let mut state = self.lock();
        if state.stopping {
            return Err(Fault::Closed);
        }
        if let Some(id) = state.owned.get(&connection).copied() {
            drop(state);
            return self.register(id, tool, handler);
        }
        if state.servers.len() == 16 {
            return Err(Fault::LimitExceeded("Servers", 16));
        }
        let mut catalog = Catalog::new(Default::default())?;
        catalog.register(tool, 1)?;
        let id = identity()?;
        state.servers.insert(
            id,
            Entry {
                catalog: Some(catalog),
                server: None,
                handlers: BTreeMap::from([(1, handler)]),
                next_handler: 1,
                closed: false,
            },
        );
        state.owned.insert(connection, id);
        Ok(())
    }
    pub(in crate::vm) fn release_owned(&self, connection: u64) {
        let id = self.lock().owned.remove(&connection);
        if let Some(id) = id {
            let _ = self.close(id);
            self.finish_close(id, false, true);
        }
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub(super) fn create(&self, limits: fpas_mcp::Limits) -> Result<u64, Fault> {
        let catalog = Catalog::new(limits)?;
        let mut state = self.lock();
        if state.stopping {
            return Err(Fault::Closed);
        }
        if state.servers.len() == 16 {
            return Err(Fault::LimitExceeded("Servers", 16));
        }
        let id = identity()?;
        state.servers.insert(
            id,
            Entry {
                catalog: Some(catalog),
                server: None,
                handlers: BTreeMap::new(),
                next_handler: 0,
                closed: false,
            },
        );
        Ok(id)
    }
    pub(super) fn register(&self, id: u64, tool: Tool, handler: RootValues) -> Result<(), Fault> {
        let mut state = self.lock();
        if state.stopping {
            return Err(Fault::Closed);
        }
        let entry = state.servers.get_mut(&id).ok_or(Fault::Closed)?;
        if entry.closed {
            return Err(Fault::Closed);
        }
        let catalog = entry.catalog.as_mut().ok_or(Fault::Busy)?;
        let key = entry.next_handler.checked_add(1).ok_or(Fault::Closed)?;
        catalog.register(tool, key)?;
        entry.next_handler = key;
        entry.handlers.insert(key, handler);
        Ok(())
    }
    pub(super) fn server(&self, id: u64) -> Result<Server, Fault> {
        let mut state = self.lock();
        if state.stopping {
            return Err(Fault::Closed);
        }
        let entry = state.servers.get_mut(&id).ok_or(Fault::Closed)?;
        if entry.closed {
            return Err(Fault::Closed);
        }
        if entry.server.is_none() {
            entry.server = entry.catalog.take().map(Server::new);
        }
        let server = entry.server.as_ref().ok_or(Fault::Closed)?;
        if server.is_closed() {
            return Err(Fault::Closed);
        }
        Ok(server.clone())
    }
    pub(super) fn handlers(&self, id: u64) -> Result<Vec<Value>, Fault> {
        let state = self.lock();
        let entry = state.servers.get(&id).ok_or(Fault::Closed)?;
        if entry.closed {
            return Err(Fault::Closed);
        }
        Ok(entry
            .handlers
            .values()
            .map(|roots| roots.with_values(|v| v[0].clone()))
            .collect())
    }
    pub(super) fn handler(&self, id: u64, key: u64) -> Result<Value, Fault> {
        let state = self.lock();
        let entry = state.servers.get(&id).ok_or(Fault::Closed)?;
        if entry.closed {
            return Err(Fault::Closed);
        }
        entry
            .handlers
            .get(&key)
            .map(|roots| roots.with_values(|v| v[0].clone()))
            .ok_or(Fault::Closed)
    }
    pub(super) fn runtime(&self) -> Result<Arc<HttpRuntime>, fpas_mcp::HttpError> {
        let mut state = self.lock();
        if state.stopping {
            return Err(Fault::Closed.into());
        }
        if let Some(runtime) = &state.runtime {
            return Ok(runtime.clone());
        }
        let runtime = Arc::new(HttpRuntime::new()?);
        state.runtime = Some(runtime.clone());
        Ok(runtime)
    }
    pub(super) fn publish(
        &self,
        id: u64,
        options: HttpOptions,
    ) -> Result<PublishTicket, super::Failure> {
        let server = self.server(id)?;
        Ok(self.runtime()?.publish(server, options)?)
    }
    pub(super) fn published(&self, id: u64, publication: Arc<Publication>) -> Result<u64, Fault> {
        let mut state = self.lock();
        if state.stopping || state.servers.get(&id).is_none_or(|s| s.closed) {
            publication.stop();
            return Err(Fault::Closed);
        }
        state
            .publications
            .retain(|_, p| p.publication.cleanup_result().is_none());
        let handle = identity()?;
        state.publications.insert(
            handle,
            Published {
                server: id,
                publication,
            },
        );
        Ok(handle)
    }
    pub(super) fn publication(&self, id: u64) -> Result<Arc<Publication>, Fault> {
        self.lock()
            .publications
            .get(&id)
            .map(|p| p.publication.clone())
            .ok_or(Fault::Closed)
    }
    pub(super) fn close_endpoint(&self, id: u64) -> Result<(bool, Vec<CleanupReceipt>), Fault> {
        let publication = self.publication(id)?;
        Ok((publication.stop(), vec![publication.receipt()]))
    }
    pub(super) fn close(&self, id: u64) -> Result<(bool, Vec<CleanupReceipt>), Fault> {
        let mut state = self.lock();
        let entry = state.servers.get_mut(&id).ok_or(Fault::Closed)?;
        let first = !entry.closed;
        entry.closed = true;
        let server = entry.server.clone();
        let receipts = match (server, state.runtime.as_ref()) {
            (Some(server), Some(runtime)) => runtime.stop_server(&server).1,
            (Some(server), None) => {
                server.close();
                Vec::new()
            }
            _ => Vec::new(),
        };
        Ok((first, receipts))
    }
    pub(super) fn finish_close(&self, id: u64, endpoint: bool, release: bool) {
        let removed = {
            let mut state = self.lock();
            if endpoint {
                state.publications.remove(&id);
                return;
            }
            state
                .publications
                .retain(|_, p| p.server != id || p.publication.cleanup_result().is_none());
            if release {
                state.servers.remove(&id)
            } else {
                None
            }
        };
        drop(removed);
    }
    pub(super) fn claim(
        &self,
        server: u64,
        responder: fpas_mcp::Responder,
    ) -> Result<Arc<Claim>, Fault> {
        let mut state = self.lock();
        if state.stopping
            || state.servers.get(&server).is_none_or(|s| s.closed)
            || responder.is_cancelled()
        {
            return Err(Fault::Closed);
        }
        let id = identity()?;
        let claim = Arc::new(Claim {
            id,
            server,
            responder,
            state: Arc::downgrade(&self.state),
        });
        state.claims.insert(id, Arc::downgrade(&claim));
        Ok(claim)
    }
    pub(super) fn is_cancelled(&self, server: u64, id: u64) -> bool {
        let claim = { self.lock().claims.get(&id).and_then(Weak::upgrade) };
        claim.is_none_or(|c| c.server != server || c.responder.is_cancelled())
    }
    pub(in crate::vm) fn shutdown(&self) {
        let (servers, runtime) = {
            let mut state = self.lock();
            state.stopping = true;
            (
                state
                    .servers
                    .values()
                    .filter_map(|s| s.server.clone())
                    .collect::<Vec<_>>(),
                state.runtime.clone(),
            )
        };
        for server in servers {
            server.close();
        }
        if let Some(runtime) = runtime {
            runtime.request_shutdown();
        }
    }
    pub(in crate::vm) fn wait_idle(&self) {
        let runtime = self.lock().runtime.clone();
        if let Some(runtime) = runtime {
            runtime.wait_closed();
        }
    }
}
impl Drop for Registry {
    fn drop(&mut self) {
        self.shutdown();
        self.wait_idle();
    }
}
