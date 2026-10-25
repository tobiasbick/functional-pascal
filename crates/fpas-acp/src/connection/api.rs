//! Synchronous admission and polling used by a cooperative language scheduler.

use super::{
    Event, Fault, Lifecycle, Payload, RequestToken, Responder,
    shared::{Command, Pending, REQUEST_TAG, RESPONDER_TAG, SERIAL_MASK, Shared},
};
use crate::{RawJson, binding, schema::Version};
use std::sync::Arc;

/// One connection owned by a native runtime. Copies alias all resource rights.
#[derive(Clone)]
pub struct Connection {
    pub(super) shared: Arc<Shared>,
}

/// Exclusive receive ownership, retained across a cooperative VM suspension.
pub struct Reader {
    shared: Arc<Shared>,
}

impl Drop for Reader {
    fn drop(&mut self) {
        self.shared.lock().reader = false;
        self.shared.changed.notify_waiters();
    }
}

impl Connection {
    /// Globally unique native identity, including the handle family tag.
    pub fn id(&self) -> u64 {
        self.shared.id
    }
    /// Selected payload schema. Negotiation is an explicit higher-level operation.
    pub fn version(&self) -> Version {
        self.shared.version
    }
    /// Detached lifecycle snapshot.
    pub fn state(&self) -> Result<Lifecycle, Fault> {
        let state = self.shared.lock();
        if state.released {
            Err(Fault::Closed)
        } else {
            Ok(state.lifecycle)
        }
    }
    /// Whether physical cleanup and SDK shutdown have completed.
    pub fn is_closed(&self) -> bool {
        self.shared.lock().closed.is_some()
    }

    /// Report failed physical cleanup without releasing its retained ownership slot.
    pub fn cleanup_result(&self) -> Result<(), Fault> {
        let state = self.shared.lock();
        let closed = state.closed.as_ref().ok_or(Fault::Busy("Close"))?;
        match &closed.cleanup_fault {
            Some(fault) => Err(fault.clone()),
            None => Ok(()),
        }
    }

    /// Admit one schema-validated request, without waiting for a peer response.
    pub fn send(&self, method: &str, raw: RawJson) -> Result<RequestToken, Fault> {
        let bytes = self.validate("ClientRequest", method, &raw)?;
        let mut state = self.shared.lock();
        Shared::active(&state)?;
        if state.pending.len() == self.shared.limits.outgoing_requests {
            return Err(Fault::LimitExceeded(
                "OutgoingRequests",
                self.shared.limits.outgoing_requests,
            ));
        }
        if state.request_serial == SERIAL_MASK {
            return Err(Fault::LimitExceeded("RequestTokens", SERIAL_MASK as usize));
        }
        let token = RequestToken(self.shared.identity(REQUEST_TAG, state.request_serial + 1));
        let metadata = self
            .shared
            .outgoing_metadata
            .reserve(method.len().saturating_mul(2).saturating_add(512))?;
        if !self.shared.commands.push(
            Command::Send {
                token,
                method: method.into(),
                raw,
            },
            bytes,
        )? {
            return Err(Fault::LimitExceeded(
                "CommandQueue",
                self.shared.limits.queue_bytes,
            ));
        }
        state.request_serial += 1;
        state.pending.insert(
            token,
            Pending {
                _metadata: metadata,
                method: method.into(),
                wire: None,
                cancellation: None,
                cancel_requested: false,
            },
        );
        drop(state);
        self.shared.changed.notify_waiters();
        Ok(token)
    }

    /// Admit one typed notification. Queue-full failures publish nothing.
    pub fn notify(&self, method: &str, raw: RawJson) -> Result<bool, Fault> {
        let bytes = self.validate("ClientNotification", method, &raw)?;
        let state = self.shared.lock();
        Shared::active(&state)?;
        if !self.shared.commands.push(
            Command::Notify {
                method: method.into(),
                raw,
            },
            bytes,
        )? {
            return Err(Fault::LimitExceeded(
                "CommandQueue",
                self.shared.limits.queue_bytes,
            ));
        }
        drop(state);
        self.shared.changed.notify_waiters();
        Ok(true)
    }

    /// Validate a successful reply before consuming the deferred response right.
    pub fn respond(&self, responder: Responder, raw: RawJson) -> Result<bool, Fault> {
        self.reply(responder, raw, false)
    }
    /// Validate a protocol error before consuming the deferred response right.
    pub fn respond_error(&self, responder: Responder, raw: RawJson) -> Result<bool, Fault> {
        self.reply(responder, raw, true)
    }

    fn reply(&self, token: Responder, raw: RawJson, error: bool) -> Result<bool, Fault> {
        let (method, id_bytes) = {
            let state = self.shared.lock();
            self.check_responder(&state, token)?;
            let Some(entry) = state.responders.get(&token) else {
                return Err(Fault::Closed);
            };
            if entry.queued {
                return Err(Fault::Closed);
            }
            (
                entry.sdk.method().to_owned(),
                serde_json::to_string(entry.sdk.id())
                    .map_err(|_| Fault::InvalidMessage)?
                    .len(),
            )
        };
        if error {
            crate::codec::decode::<agent_client_protocol::Error>(raw.clone())?;
        } else {
            self.validate("ClientResponse", &method, &raw)
                .map_err(|_| Fault::InvalidArgument("Response"))?;
        }
        if raw.text().len().saturating_add(id_bytes).saturating_add(64)
            > self.shared.limits.message_bytes
        {
            return Err(Fault::LimitExceeded(
                "MessageBytes",
                self.shared.limits.message_bytes,
            ));
        }
        let value: serde_json::Value =
            serde_json::from_str(raw.text()).map_err(|_| Fault::InvalidMessage)?;
        let bytes = raw
            .text()
            .len()
            .saturating_add(super::budget::json_size(&value).saturating_mul(2))
            .saturating_add(id_bytes)
            .saturating_add(128);
        let mut state = self.shared.lock();
        Shared::active(&state)?;
        let Some(entry) = state.responders.get_mut(&token) else {
            return Err(Fault::Closed);
        };
        if entry.queued {
            return Err(Fault::Closed);
        }
        if !self.shared.commands.push(
            Command::Reply {
                responder: token,
                raw,
                error,
            },
            bytes,
        )? {
            return Err(Fault::LimitExceeded(
                "CommandQueue",
                self.shared.limits.queue_bytes,
            ));
        }
        entry.queued = true;
        drop(state);
        self.shared.changed.notify_waiters();
        Ok(true)
    }

    /// Request protocol cancellation at most once; completed tokens return false.
    pub fn cancel(&self, token: RequestToken) -> Result<bool, Fault> {
        let mut state = self.shared.lock();
        if !self
            .shared
            .valid(token.0, REQUEST_TAG, state.request_serial)
            || state.released
        {
            return Err(Fault::Closed);
        }
        let Some(pending) = state.pending.get_mut(&token) else {
            return Ok(false);
        };
        if pending.cancel_requested {
            return Ok(false);
        }
        if Shared::active(&state).is_err() {
            return Ok(false);
        }
        match self.shared.commands.push(Command::Cancel { token }, 128) {
            Ok(true) => {}
            Ok(false) => {
                return Err(Fault::LimitExceeded(
                    "CommandQueue",
                    self.shared.limits.queue_bytes,
                ));
            }
            Err(Fault::Closed) => return Ok(false),
            Err(error) => return Err(error),
        }
        let pending = state.pending.get_mut(&token).ok_or(Fault::Closed)?;
        pending.cancel_requested = true;
        drop(state);
        self.shared.changed.notify_waiters();
        Ok(true)
    }

    /// Check a native token representation against its issuing connection.
    pub fn request_token(&self, id: u64) -> Result<RequestToken, Fault> {
        let state = self.shared.lock();
        if state.released || !self.shared.valid(id, REQUEST_TAG, state.request_serial) {
            return Err(Fault::Closed);
        }
        Ok(RequestToken(id))
    }

    /// Check a deferred responder representation against its issuing connection.
    pub fn responder(&self, id: u64) -> Result<Responder, Fault> {
        let state = self.shared.lock();
        self.check_responder(&state, Responder(id))?;
        Ok(Responder(id))
    }

    /// Method bound to an active response right, before payload validation.
    pub fn responder_method(&self, token: Responder) -> Result<String, Fault> {
        let state = self.shared.lock();
        self.check_responder(&state, token)?;
        let entry = state
            .responders
            .get(&token)
            .filter(|entry| !entry.queued)
            .ok_or(Fault::Closed)?;
        Ok(entry.sdk.method().into())
    }

    fn check_responder(&self, state: &super::shared::State, token: Responder) -> Result<(), Fault> {
        if state.released
            || !self
                .shared
                .valid(token.0, RESPONDER_TAG, state.responder_serial)
        {
            return Err(Fault::Closed);
        }
        Ok(())
    }

    /// Reserve the only active event reader, including while it is suspended.
    pub fn reader(&self) -> Result<Reader, Fault> {
        let mut state = self.shared.lock();
        if state.released {
            return Err(Fault::Closed);
        }
        if state.reader {
            return Err(Fault::Busy("Receive"));
        }
        state.reader = true;
        Ok(Reader {
            shared: self.shared.clone(),
        })
    }

    /// Consume one event without retaining receive ownership.
    pub fn try_receive(&self) -> Result<Option<Event>, Fault> {
        self.reader()?.try_receive()
    }

    /// Begin idempotent cooperative close. Poll is_closed before completing cleanup.
    pub fn close(&self) -> Result<bool, Fault> {
        let mut state = self.shared.lock();
        if state.released {
            return Err(Fault::Closed);
        }
        if matches!(
            state.lifecycle,
            Lifecycle::Closing | Lifecycle::Closed | Lifecycle::Failed
        ) {
            return Ok(false);
        }
        state.lifecycle = Lifecycle::Closing;
        state.local_close = true;
        drop(state);
        self.shared.changed.notify_waiters();
        Ok(true)
    }

    /// Invalidate aliases after cleanup and discard unread terminal events.
    pub fn release(&self) -> Result<bool, Fault> {
        let mut state = self.shared.lock();
        if state.released {
            return Err(Fault::Closed);
        }
        if state.closed.is_none() {
            return Err(Fault::Busy("Close"));
        }
        if let Some(fault) = state
            .closed
            .as_ref()
            .and_then(|closed| closed.cleanup_fault.clone())
        {
            return Err(fault);
        }
        state.released = true;
        state.pending.clear();
        drop(state);
        self.shared.events.discard();
        self.shared.changed.notify_waiters();
        Ok(true)
    }

    fn validate(&self, root: &str, method: &str, raw: &RawJson) -> Result<usize, Fault> {
        let method_bytes = serde_json::to_string(method)
            .map_err(|_| Fault::InvalidMessage)?
            .len();
        if raw
            .text()
            .len()
            .saturating_add(method_bytes)
            .saturating_add(128)
            > self.shared.limits.message_bytes
        {
            return Err(Fault::LimitExceeded(
                "MessageBytes",
                self.shared.limits.message_bytes,
            ));
        }
        binding::catalog(self.shared.version)?.decode_message(root, method, raw.clone())?;
        let value: serde_json::Value =
            serde_json::from_str(raw.text()).map_err(|_| Fault::InvalidMessage)?;
        if root != "ClientResponse" {
            agent_client_protocol::RawJsonRpcParams::from_value(value.clone())
                .map_err(|_| Fault::InvalidMessage)?;
        }
        Ok(raw
            .text()
            .len()
            .saturating_add(super::budget::json_size(&value).saturating_mul(2))
            .saturating_add(method_bytes)
            .saturating_add(256))
    }
}

impl Reader {
    /// Consume an admitted event, then reserved failures, then exactly one Closed.
    pub fn try_receive(&self) -> Result<Option<Event>, Fault> {
        let mut state = self.shared.lock();
        if state.released {
            return Err(Fault::Closed);
        }
        if let Some(item) = self.shared.events.pop() {
            return Ok(Some(item.into_value()));
        }
        let Some(closed) = state.closed.clone() else {
            return Ok(None);
        };
        if let Some((token, _)) = state.pending.pop_first() {
            return Ok(Some(Event {
                payload: Payload::RequestFailed(token, closed.fault.unwrap_or(Fault::Closed)),
                raw: None,
            }));
        }
        if state.closed_delivered {
            return Err(Fault::Closed);
        }
        state.closed_delivered = true;
        Ok(Some(Event {
            payload: Payload::Closed(closed),
            raw: None,
        }))
    }

    /// Wait for native state changes; VM integrations normally poll cooperatively.
    pub async fn receive(&self) -> Result<Event, Fault> {
        loop {
            let changed = self.shared.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if let Some(event) = self.try_receive()? {
                return Ok(event);
            }
            changed.await;
        }
    }
}
