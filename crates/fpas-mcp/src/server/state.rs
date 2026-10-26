//! Shared admission state; removing a call releases both queue and payload budgets.

use crate::{
    Catalog, Fault,
    model::{CallToolResult, ErrorData, JsonObject, ProtocolVersion, RequestId, RequestMetaObject},
};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Mutex, MutexGuard},
};
use tokio::sync::{Notify, oneshot};
use tokio_util::sync::CancellationToken;

pub(super) struct Shared {
    pub catalog: Catalog,
    pub state: Mutex<State>,
    pub changed: Notify,
}

#[derive(Default)]
pub(super) struct State {
    pub closed: bool,
    pub reader: bool,
    pub next_call: u64,
    pub next_endpoint: u64,
    pub endpoints: BTreeMap<u64, EndpointState>,
    pub pending: BTreeMap<u64, Pending>,
    pub queue: VecDeque<u64>,
    pub bytes: usize,
}

pub(super) struct Data {
    pub connection_id: Option<u64>,
    pub session_id: Option<String>,
    pub handler: u64,
    pub name: String,
    pub arguments: JsonObject,
    pub request_id: RequestId,
    pub metadata: RequestMetaObject,
    pub protocol_version: Option<ProtocolVersion>,
}

pub(super) struct EndpointState {
    pub cancellation: CancellationToken,
    pub binding: Binding,
}

pub(super) enum Binding {
    Standalone,
    Pending(u64),
    Session(u64, String),
}

pub(super) struct Pending {
    pub endpoint: u64,
    pub name: String,
    pub data: Option<Data>,
    pub sender: oneshot::Sender<Result<CallToolResult, ErrorData>>,
    pub cancellation: CancellationToken,
    pub bytes: usize,
}

impl Shared {
    pub fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }
}

impl State {
    pub fn active(&self) -> Result<(), Fault> {
        if self.closed {
            Err(Fault::Closed)
        } else {
            Ok(())
        }
    }

    pub fn take(&mut self, id: u64) -> Option<Pending> {
        let pending = self.pending.remove(&id)?;
        self.queue.retain(|queued| *queued != id);
        self.bytes -= pending.bytes;
        Some(pending)
    }

    pub fn discard(&mut self, id: u64) {
        if let Some(pending) = self.take(id) {
            let _ = pending
                .sender
                .send(Err(ErrorData::internal_error("MCP call is closed", None)));
        }
    }
}

pub(super) struct CallGuard<'a> {
    pub shared: &'a Shared,
    pub id: u64,
}

impl Drop for CallGuard<'_> {
    fn drop(&mut self) {
        self.shared.lock().discard(self.id);
        self.shared.changed.notify_waiters();
    }
}
