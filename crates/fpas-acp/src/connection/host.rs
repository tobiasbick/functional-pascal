//! A host execution lease is independent of the SDK's single reply right.

use super::{Connection, Fault, Responder, shared::Shared};
use crate::{RawJson, binding};
use std::sync::Arc;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Execution {
    #[default]
    Available,
    Running,
    Finished,
}

/// One admitted host effect. Dropping it finishes the execution even on failure.
pub struct HostClaim {
    shared: Arc<Shared>,
    responder: Responder,
}
impl Drop for HostClaim {
    fn drop(&mut self) {
        if let Some(entry) = self.shared.lock().responders.get_mut(&self.responder) {
            entry.execution = Execution::Finished;
        }
        self.shared.changed.notify_waiters();
    }
}
impl Connection {
    /// Protocol family of the original incoming request, independent of later negotiation.
    pub fn responder_version(&self, responder: Responder) -> Result<crate::schema::Version, Fault> {
        let state = self.shared.lock();
        Shared::active(&state)?;
        let entry = state.responders.get(&responder).ok_or(Fault::Closed)?;
        if entry.queued || entry.cancellation.is_cancelled() {
            return Err(Fault::Closed);
        }
        Ok(entry.version)
    }
    /// Validate identity and exact typed arguments before atomically claiming one host execution.
    pub fn claim_host(
        &self,
        responder: Responder,
        method: &str,
        request: &RawJson,
    ) -> Result<HostClaim, Fault> {
        self.claim_host_with(responder, method, request, || Ok(()))
            .map(|(claim, ())| claim)
    }
    /// Reserve native resources under the same lock as request/session admission.
    pub fn claim_host_with<T>(
        &self,
        responder: Responder,
        method: &str,
        request: &RawJson,
        reserve: impl FnOnce() -> Result<T, Fault>,
    ) -> Result<(HostClaim, T), Fault> {
        let mut state = self.shared.lock();
        Shared::active(&state)?;
        let entry = state.responders.get_mut(&responder).ok_or(Fault::Closed)?;
        if entry.queued || entry.cancellation.is_cancelled() {
            return Err(Fault::Closed);
        }
        if entry.sdk.method() != method {
            return Err(Fault::InvalidArgument("HostRequest"));
        }
        let catalog = binding::catalog(self.version())?;
        if catalog
            .decode_message("AgentRequest", method, entry.params.clone())?
            .payload
            != catalog
                .decode_message("AgentRequest", method, request.clone())?
                .payload
        {
            return Err(Fault::InvalidArgument("HostRequest"));
        }
        match entry.execution {
            Execution::Running => return Err(Fault::Busy("HostExecution")),
            Execution::Finished => return Err(Fault::InvalidArgument("HostExecution")),
            Execution::Available => {}
        }
        if let Some(session) = request.member("sessionId")? {
            let session: String = crate::codec::deserialize(&session)?;
            if !state.client.has_session(&session) {
                return Err(Fault::Closed);
            }
        }
        let reservation = reserve()?;
        let entry = state.responders.get_mut(&responder).ok_or(Fault::Closed)?;
        entry.execution = Execution::Running;
        Ok((
            HostClaim {
                shared: self.shared.clone(),
                responder,
            },
            reservation,
        ))
    }
    /// Return the actual incoming payload only while its interaction is active.
    pub fn incoming(&self, responder: Responder, method: &str) -> Result<RawJson, Fault> {
        let state = self.shared.lock();
        Shared::active(&state)?;
        let entry = state.responders.get(&responder).ok_or(Fault::Closed)?;
        if entry.queued || entry.cancellation.is_cancelled() {
            return Err(Fault::Closed);
        }
        if entry.sdk.method() != method {
            return Err(Fault::InvalidArgument("HostRequest"));
        }
        Ok(entry.params.clone())
    }
    /// Validate a form/URL response against its live original request without consuming it.
    pub fn validate_interaction(
        &self,
        responder: Responder,
        response: &RawJson,
    ) -> Result<bool, Fault> {
        let request = self.incoming(responder, "elicitation/create")?;
        crate::interaction::validate("elicitation/create", &request, response)?;
        Ok(true)
    }
    /// Configured resource limits for explicitly invoked host adapters.
    pub fn limits(&self) -> &super::Limits {
        &self.shared.limits
    }
}
