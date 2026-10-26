//! Bounded SDK-to-application calls with independent attachment identities.

mod adapter;
mod reply;
mod state;

use crate::{
    Catalog, Fault,
    model::{JsonObject, ProtocolVersion, RequestId, RequestMetaObject},
};
pub use reply::Responder;
use state::{Binding, EndpointState, Shared, State};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Published catalog and application queue. Clones alias the same server.
#[derive(Clone)]
pub struct Server {
    shared: Arc<Shared>,
    identity: Arc<()>,
}

/// One application or ACP attachment to a published server.
/// Clones alias its authority; closing any clone invalidates all of them.
#[derive(Clone)]
pub struct Endpoint {
    shared: Arc<Shared>,
    id: u64,
    cancellation: CancellationToken,
}

/// Exclusive receive ownership retained across cooperative waits.
pub struct Reader {
    shared: Arc<Shared>,
}

/// Validated arguments and context detached from the SDK executor.
/// Receiving a call does not grant application permission to perform effects.
pub struct Call {
    /// Transport-owner identity for an attached agent connection.
    pub connection_id: Option<u64>,
    /// Session bound before this call was admitted; never redirected afterward.
    pub session_id: Option<String>,
    /// Embedding application's registration identity.
    pub handler: u64,
    /// Registered tool name.
    pub name: String,
    /// Arguments validated against the catalog's input schema.
    pub arguments: JsonObject,
    /// SDK-assigned remote request identity, distinct from the local responder.
    pub request_id: RequestId,
    /// Original protocol metadata, without a retained SDK peer.
    pub metadata: RequestMetaObject,
    /// Negotiated or explicitly supplied protocol revision.
    pub protocol_version: Option<ProtocolVersion>,
    /// One-use local response right and cancellation observation.
    pub responder: Responder,
}

impl Server {
    pub(crate) fn identity(&self) -> std::sync::Weak<()> {
        Arc::downgrade(&self.identity)
    }
    /// Observe terminal publication state without allocating a reader.
    pub fn is_closed(&self) -> bool {
        self.shared.lock().closed
    }
    pub(crate) fn limits(&self) -> &crate::Limits {
        &self.shared.catalog.limits
    }
    /// Freeze a complete catalog. Publishing consumes the registration builder.
    pub fn new(catalog: Catalog) -> Self {
        Self {
            identity: Arc::new(()),
            shared: Arc::new(Shared {
                catalog,
                state: std::sync::Mutex::new(State::default()),
                changed: tokio::sync::Notify::new(),
            }),
        }
    }

    /// Create an independently cancellable attachment without granting host access.
    pub fn endpoint(&self) -> Result<Endpoint, Fault> {
        let mut state = self.shared.lock();
        state.active()?;
        let maximum = self.shared.catalog.limits.endpoints;
        if state.endpoints.len() == maximum {
            return Err(Fault::LimitExceeded("Endpoints", maximum));
        }
        state.next_endpoint = state.next_endpoint.checked_add(1).ok_or(Fault::Closed)?;
        let id = state.next_endpoint;
        let cancellation = CancellationToken::new();
        state.endpoints.insert(
            id,
            EndpointState {
                cancellation: cancellation.clone(),
                binding: Binding::Standalone,
            },
        );
        Ok(Endpoint {
            shared: self.shared.clone(),
            id,
            cancellation,
        })
    }

    /// Acquire the only application reader; another owner gets `Busy`.
    pub fn reader(&self) -> Result<Reader, Fault> {
        let mut state = self.shared.lock();
        state.active()?;
        if state.reader {
            return Err(Fault::Busy);
        }
        state.reader = true;
        Ok(Reader {
            shared: self.shared.clone(),
        })
    }

    /// Stop admission, cancel endpoints and settle every retained response right.
    /// The caller still owns physical transport shutdown.
    pub fn close(&self) -> bool {
        let mut state = self.shared.lock();
        if state.closed {
            return false;
        }
        state.closed = true;
        for token in state.endpoints.values() {
            token.cancellation.cancel();
        }
        state.endpoints.clear();
        let ids = state.pending.keys().copied().collect::<Vec<_>>();
        for id in ids {
            state.discard(id);
        }
        self.shared.changed.notify_waiters();
        true
    }
}

impl Endpoint {
    /// Reserve a fresh attachment before advertising its confidential access.
    pub fn begin_attachment(&self, connection: u64) -> Result<(), Fault> {
        let mut state = self.shared.lock();
        state.active()?;
        if state.pending.values().any(|call| call.endpoint == self.id) {
            return Err(Fault::Busy);
        }
        let endpoint = state.endpoints.get_mut(&self.id).ok_or(Fault::Closed)?;
        if !matches!(endpoint.binding, Binding::Standalone) {
            return Err(Fault::Busy);
        }
        endpoint.binding = Binding::Pending(connection);
        Ok(())
    }

    /// Activate one pending attachment. An established identity cannot be rebound.
    pub fn bind_session(&self, session: &str) -> Result<(), Fault> {
        if session.is_empty() || session.len() > self.shared.catalog.limits.message_bytes {
            return Err(Fault::InvalidArgument("SessionId"));
        }
        let mut state = self.shared.lock();
        state.active()?;
        let endpoint = state.endpoints.get_mut(&self.id).ok_or(Fault::Closed)?;
        let Binding::Pending(connection) = endpoint.binding else {
            return Err(Fault::InvalidArgument("Attachment"));
        };
        endpoint.binding = Binding::Session(connection, session.to_owned());
        Ok(())
    }
    /// Server-local attachment identity; a replacement always has another identity.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Observe attachment or whole-server cancellation from transport ownership code.
    pub async fn cancelled(&self) {
        self.cancellation.cancelled().await;
    }

    /// Retire this attachment without closing the caller-owned server or other routes.
    pub fn close(&self) -> bool {
        let mut state = self.shared.lock();
        let Some(token) = state.endpoints.remove(&self.id) else {
            return false;
        };
        token.cancellation.cancel();
        let ids = state
            .pending
            .iter()
            .filter_map(|(id, pending)| (pending.endpoint == self.id).then_some(*id))
            .collect::<Vec<_>>();
        for id in ids {
            state.discard(id);
        }
        self.shared.changed.notify_waiters();
        true
    }
}

impl Reader {
    /// Claim one ready call without blocking. Cancelled queued calls are skipped.
    pub fn try_receive(&mut self) -> Result<Option<Call>, Fault> {
        let mut state = self.shared.lock();
        state.active()?;
        while let Some(id) = state.queue.pop_front() {
            let Some(pending) = state.pending.get_mut(&id) else {
                continue;
            };
            if pending.cancellation.is_cancelled() || pending.sender.is_closed() {
                state.discard(id);
                continue;
            }
            let Some(data) = pending.data.take() else {
                continue;
            };
            return Ok(Some(Call {
                connection_id: data.connection_id,
                session_id: data.session_id,
                handler: data.handler,
                name: data.name,
                arguments: data.arguments,
                request_id: data.request_id,
                metadata: data.metadata,
                protocol_version: data.protocol_version,
                responder: Responder::new(self.shared.clone(), id),
            }));
        }
        Ok(None)
    }

    /// Await one ready application call without executing callbacks on the SDK task.
    pub async fn receive(&mut self) -> Result<Call, Fault> {
        let shared = self.shared.clone();
        loop {
            let notified = shared.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if let Some(call) = self.try_receive()? {
                return Ok(call);
            }
            notified.await;
        }
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        self.shared.lock().reader = false;
        self.shared.changed.notify_waiters();
    }
}
