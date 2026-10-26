//! Native admission state. SDK calls and transport cleanup happen outside its lock.

use super::{
    Closed, Event, Fault, Lifecycle, Limits, Payload, RequestToken, Responder, queue::Queue,
};
use crate::{RawJson, binding::Value, protocol::v1::RequestId, schema::Version};
use agent_client_protocol::{RequestCancellation, RequestCancellationHandle};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex, MutexGuard},
};

pub(super) const CONNECTION_TAG: u64 = 0x4143_0000_0000_0000;
pub(super) const REQUEST_TAG: u64 = 0x4144_0000_0000_0000;
pub(super) const RESPONDER_TAG: u64 = 0x4146_0000_0000_0000;
pub(super) const SERIAL_MASK: u64 = 0xff_ffff;

pub(super) enum Command {
    RetireRoute {
        tokens: Vec<RequestToken>,
        responders: Vec<Responder>,
    },
    Cancel {
        token: RequestToken,
    },
    Send {
        token: RequestToken,
        method: String,
        raw: RawJson,
    },
    Notify {
        method: String,
        raw: RawJson,
        cancelled: Vec<Responder>,
    },
    Reply {
        responder: Responder,
        raw: RawJson,
        error: bool,
    },
}

pub(super) struct Pending {
    pub version: Version,
    pub action: super::client::Action,
    pub _metadata: super::budget::Permit,
    pub method: String,
    pub wire: Option<RequestId>,
    pub cancellation: Option<RequestCancellationHandle>,
    pub cancel_requested: bool,
}

pub(super) struct Deferred {
    pub version: Version,
    pub route: Option<u64>,
    pub params: RawJson,
    pub execution: super::host::Execution,
    pub session: Option<String>,
    pub _metadata: super::budget::Permit,
    pub sdk: agent_client_protocol::Responder<serde_json::Value>,
    pub cancellation: RequestCancellation,
    pub queued: bool,
}

pub(super) enum Publication {
    Cancellation(RequestId),
    Request(RequestId),
    Notification(String),
    Response(RequestId),
}

pub(super) struct State {
    pub routes: BTreeMap<String, super::mcp::Endpoint>,
    pub route_serial: u64,
    pub client: super::client::Client,
    pub cancelled_replies: BTreeMap<RequestId, super::budget::Permit>,
    pub local_close: bool,
    pub lifecycle: Lifecycle,
    pub fault: Option<Fault>,
    pub closed: Option<Closed>,
    pub closed_delivered: bool,
    pub released: bool,
    pub reader: bool,
    pub request_serial: u64,
    pub responder_serial: u64,
    pub pending: BTreeMap<RequestToken, Pending>,
    pub responders: BTreeMap<Responder, Deferred>,
    pub publications: VecDeque<(Publication, Box<dyn Send>)>,
}

pub(super) struct Shared {
    pub outgoing_metadata: Arc<super::budget::Budget>,
    pub incoming_metadata: Arc<super::budget::Budget>,
    pub id: u64,
    pub version: Version,
    pub family: std::sync::atomic::AtomicU8,
    pub limits: Limits,
    pub state: Mutex<State>,
    pub commands: Arc<Queue<Command>>,
    pub events: Arc<Queue<Event>>,
    pub changed: tokio::sync::Notify,
}

impl Shared {
    pub fn new(id: u64, settings: super::Settings, limits: Limits) -> Result<Arc<Self>, Fault> {
        let version = settings.requested_version()?;
        let client = super::client::Client::new(settings, &limits)?;
        Ok(Arc::new(Self {
            outgoing_metadata: super::budget::Budget::new("OutgoingMetadata", limits.queue_bytes),
            incoming_metadata: super::budget::Budget::new("IncomingMetadata", limits.queue_bytes),
            id,
            version,
            family: std::sync::atomic::AtomicU8::new(if version == Version::V1 { 1 } else { 2 }),
            commands: Queue::new(limits.outbound_entries, limits.queue_bytes),
            events: Queue::new(limits.queue_entries, limits.queue_bytes),
            state: Mutex::new(State {
                routes: BTreeMap::new(),
                route_serial: 0,
                client,
                cancelled_replies: BTreeMap::new(),
                local_close: false,
                lifecycle: Lifecycle::Uninitialized,
                fault: None,
                closed: None,
                closed_delivered: false,
                released: false,
                reader: false,
                request_serial: 0,
                responder_serial: 0,
                pending: BTreeMap::new(),
                responders: BTreeMap::new(),
                publications: VecDeque::new(),
            }),
            limits,
            changed: tokio::sync::Notify::new(),
        }))
    }

    pub fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub fn family(&self) -> Version {
        if self.family.load(std::sync::atomic::Ordering::Acquire) == 1 {
            Version::V1
        } else {
            Version::V2
        }
    }

    pub fn active(state: &State) -> Result<(), Fault> {
        if state.released
            || matches!(
                state.lifecycle,
                Lifecycle::Closing | Lifecycle::Closed | Lifecycle::Failed
            )
        {
            Err(Fault::Closed)
        } else {
            Ok(())
        }
    }

    pub fn identity(&self, tag: u64, serial: u64) -> u64 {
        tag | ((self.id & SERIAL_MASK) << 24) | serial
    }

    pub fn valid(&self, value: u64, tag: u64, issued: u64) -> bool {
        value & !SERIAL_MASK == self.identity(tag, 0)
            && value & SERIAL_MASK != 0
            && value & SERIAL_MASK <= issued
    }

    pub fn fail(&self, fault: Fault) {
        let mut state = self.lock();
        if state.closed.is_none() && state.fault.is_none() {
            state.fault = Some(fault);
            state.lifecycle = Lifecycle::Closing;
        }
        drop(state);
        self.changed.notify_waiters();
    }

    pub fn stopping(&self) -> bool {
        matches!(
            self.lock().lifecycle,
            Lifecycle::Closing | Lifecycle::Closed | Lifecycle::Failed
        )
    }

    pub async fn wait_stopping(&self) {
        loop {
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.stopping() {
                return;
            }
            changed.await;
        }
    }

    pub fn end_input(&self) {
        let mut state = self.lock();
        if Self::active(&state).is_ok() {
            state.lifecycle = Lifecycle::Closing;
        }
        drop(state);
        self.changed.notify_waiters();
    }

    pub fn event(&self, event: Event) -> Result<(), Fault> {
        let bytes = event_size(&event);
        if !self.events.push(event, bytes)? {
            let fault = Fault::LimitExceeded("EventQueue", self.limits.queue_bytes);
            self.fail(fault.clone());
            return Err(fault);
        }
        self.changed.notify_waiters();
        Ok(())
    }

    pub fn response(&self, token: RequestToken, mut event: Event) -> Result<(), Fault> {
        let mut state = self.lock();
        let mut cancelled = Vec::new();
        let closed_sessions = match state.pending.get(&token).map(|pending| &pending.action) {
            Some(super::client::Action::Close(id))
                if matches!(event.payload, Payload::Response(_, _, _)) =>
            {
                state.client.session_tree(id)
            }
            _ => Default::default(),
        };
        {
            let response = event
                .raw
                .as_ref()
                .map(|raw| raw.member("result"))
                .transpose()?
                .flatten();
            let State {
                client, pending, ..
            } = &mut *state;
            if let Some(pending) = pending.get(&token) {
                let initialized = matches!(
                    pending.action,
                    super::client::Action::Initialize | super::client::Action::InitializeV2
                );
                if let Err(fault) = client.settle(&pending.action, token, response.as_ref()) {
                    event.payload = Payload::RequestFailed(token, fault.clone());
                    state.fault = Some(fault);
                    state.lifecycle = Lifecycle::Closing;
                } else if initialized {
                    if let Payload::RequestFailed(_, fault) = &event.payload {
                        state.fault = Some(fault.clone());
                        state.lifecycle = Lifecycle::Closing;
                    } else {
                        state.lifecycle = Lifecycle::Ready;
                        self.family.store(
                            if state.client.negotiated_version() == Some(Version::V1) {
                                1
                            } else {
                                2
                            },
                            std::sync::atomic::Ordering::Release,
                        );
                    }
                }
            }
            if !closed_sessions.is_empty() && matches!(event.payload, Payload::Response(_, _, _)) {
                cancelled = super::client::cancellation::select_closed(&state, &closed_sessions);
                super::client::cancellation::mark(&mut state, &cancelled);
                let endpoints: Vec<_> = state
                    .routes
                    .iter()
                    .filter(|(_, endpoint)| {
                        endpoint
                            .session
                            .as_ref()
                            .is_some_and(|id| closed_sessions.contains(id))
                    })
                    .map(|(key, endpoint)| (key.clone(), endpoint.id))
                    .collect();
                for (key, id) in endpoints {
                    super::mcp::retire(self, &mut state, &key, id)?;
                }
            }
        }
        let bytes = event_size(&event);
        if !self.events.push(event, bytes)? {
            let fault = Fault::LimitExceeded("EventQueue", self.limits.queue_bytes);
            state.fault = Some(fault.clone());
            state.lifecycle = Lifecycle::Closing;
            drop(state);
            self.changed.notify_waiters();
            return Err(fault);
        }
        // Readers hold the same state lock: a visible response has already
        // retired its token and returned the pending-request reservation.
        state.pending.remove(&token);
        drop(state);
        super::client::cancellation::settle(self, &cancelled)?;
        self.changed.notify_waiters();
        Ok(())
    }

    pub fn cancel_incoming(&self, token: Responder, raw: RawJson) -> Result<(), Fault> {
        let deferred = {
            let mut state = self.lock();
            if state
                .responders
                .get(&token)
                .is_none_or(|entry| entry.queued)
            {
                return Ok(());
            }
            state.responders.remove(&token)
        };
        if let Some(deferred) = deferred {
            let _ = deferred
                .sdk
                .respond_with_error(agent_client_protocol::Error::request_cancelled());
            self.event(Event {
                payload: Payload::RequestCancelled(token),
                raw: Some(raw),
            })?;
        }
        Ok(())
    }

    pub fn finish(&self, closed: Closed) {
        let mut state = self.lock();
        state.lifecycle = if closed.fault.is_some() {
            Lifecycle::Failed
        } else {
            Lifecycle::Closed
        };
        state.closed = Some(closed);
        drop(state);
        self.changed.notify_waiters();
    }

    pub fn stop_admission(&self) {
        self.commands.close();
        self.commands.discard();
        let (responders, publications) = {
            let mut state = self.lock();
            state.lifecycle = Lifecycle::Closing;
            state.client.close();
            state.routes.clear();
            state.cancelled_replies.clear();
            (
                std::mem::take(&mut state.responders),
                std::mem::take(&mut state.publications),
            )
        };
        drop((responders, publications));
        self.changed.notify_waiters();
    }
}

fn event_size(event: &Event) -> usize {
    let raw = event.raw.as_ref().map_or(0, |raw| raw.text().len());
    raw.saturating_add(match &event.payload {
        Payload::Request(_, _, value)
        | Payload::Initialized(_, _, value)
        | Payload::McpResponse(_, _, value)
        | Payload::Response(_, _, value)
        | Payload::Notification(_, value) => value_size(value),
        Payload::UnknownNotification(method, value) => {
            method.len().saturating_add(value.text().len())
        }
        Payload::RequestFailed(_, Fault::Peer(value)) => value.text().len(),
        _ => 64,
    })
    .saturating_add(std::mem::size_of::<Event>())
}

fn value_size(value: &Value) -> usize {
    std::mem::size_of::<Value>().saturating_add(match value {
        Value::String(text) => text.len(),
        Value::Raw(raw) => raw.text().len(),
        Value::Array(values) => values.iter().map(value_size).sum(),
        Value::Dictionary(values) => values
            .iter()
            .map(|(key, value)| key.len() + value_size(value))
            .sum(),
        Value::Some(value) | Value::Replacement(value) => value_size(value),
        Value::Record { name, fields } => name.len() + fields.iter().map(value_size).sum::<usize>(),
        Value::Enum {
            name,
            variant,
            fields,
        } => name.len() + variant.len() + fields.iter().map(value_size).sum::<usize>(),
        _ => 0,
    })
}
