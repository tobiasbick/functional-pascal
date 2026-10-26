//! Single-use, version-bound response admission.
use super::{
    Connection, Fault, Responder,
    shared::{Command, Shared},
};
use crate::RawJson;
impl Connection {
    /// Validate a successful reply before consuming the deferred response right.
    pub fn respond(&self, responder: Responder, raw: RawJson) -> Result<bool, Fault> {
        self.reply(responder, raw, false)
    }
    /// Validate a protocol error before consuming the deferred response right.
    pub fn respond_error(&self, responder: Responder, raw: RawJson) -> Result<bool, Fault> {
        self.reply(responder, raw, true)
    }

    fn reply(&self, token: Responder, raw: RawJson, error: bool) -> Result<bool, Fault> {
        let (method, id_bytes, version) = {
            let state = self.shared.lock();
            self.check_responder(&state, token)?;
            let Some(entry) = state.responders.get(&token) else {
                return Err(Fault::Closed);
            };
            if entry.queued {
                return Err(Fault::Closed);
            }
            if entry.cancellation.is_cancelled() {
                return Err(Fault::Closed);
            }
            if entry.execution == super::host::Execution::Running {
                return Err(Fault::Busy("HostExecution"));
            }
            if !error {
                crate::interaction::validate(entry.sdk.method(), &entry.params, &raw)?;
            }
            (
                entry.sdk.method().to_owned(),
                serde_json::to_string(entry.sdk.id())
                    .map_err(|_| Fault::InvalidMessage)?
                    .len(),
                entry.version,
            )
        };
        if error {
            crate::codec::decode::<agent_client_protocol::Error>(raw.clone())?;
        } else {
            self.validate_for(version, "ClientResponse", &method, &raw)
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
        if entry.cancellation.is_cancelled() {
            return Err(Fault::Closed);
        }
        if entry.execution == super::host::Execution::Running {
            return Err(Fault::Busy("HostExecution"));
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
}
