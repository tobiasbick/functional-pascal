//! Inbound session updates, host admission and session cancellation.
use super::super::{Fault, Lifecycle, Limits};
use super::sessions::Session;
use super::{capabilities, sessions, v1::Client};
use crate::{RawJson, protocol::v1 as sdk, schema::Version};
impl Client {
    pub fn notification(
        &mut self,
        lifecycle: Lifecycle,
        method: &str,
        raw: &RawJson,
        limits: &Limits,
    ) -> Result<(), Fault> {
        if method != "session/update" {
            return Ok(());
        }
        if lifecycle != Lifecycle::Ready {
            return Err(Fault::InvalidMessage);
        }
        let id = sessions::id(raw)?;
        if !self.sessions.contains_key(&id) {
            if self.creating.is_empty() || self.sessions.len() >= limits.sessions {
                return Err(Fault::InvalidMessage);
            }
            self.sessions
                .insert(id.clone(), Session::new(&id, &self.budget)?);
        }
        let update = raw.member("update")?.ok_or(Fault::InvalidMessage)?;
        let kind = update
            .member("sessionUpdate")?
            .ok_or(Fault::InvalidMessage)?;
        let kind: String = crate::codec::deserialize(&kind)?;
        if kind == "subagent_update" {
            self.observe_subagent(&id, &update)?;
        }
        let session = self.sessions.get_mut(&id).ok_or(Fault::InvalidMessage)?;
        match kind.as_str() {
            "config_option_update" => session.replace_version(
                &update,
                &self.budget,
                self.negotiated.unwrap_or(Version::V1),
            )?,
            "state_update" => {
                if update
                    .member("state")?
                    .is_some_and(|state| state.text() == "\"idle\"")
                {
                    session.cancelled = false;
                }
            }
            "current_mode_update" => {
                if let Some(modes) = &session.modes {
                    let update: sdk::CurrentModeUpdate = crate::codec::deserialize(&update)?;
                    let mut modes = modes.clone();
                    modes.current_mode_id = update.current_mode_id;
                    let raw = RawJson::parse(
                        &serde_json::to_string(&serde_json::json!({"modes":modes}))
                            .map_err(|_| Fault::InvalidMessage)?,
                    )?;
                    session.replace(&raw, &self.budget)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn allow_incoming(
        &mut self,
        lifecycle: Lifecycle,
        method: &str,
        raw: &RawJson,
        request_scope: bool,
    ) -> Result<bool, Fault> {
        if method.starts_with('_') {
            return Ok(true);
        }
        if method == "mcp/message" {
            return Ok(lifecycle == Lifecycle::Ready && self.settings.experimental.mcp);
        }
        if !capabilities::incoming(&self.settings.client_capabilities, method, raw) {
            return Ok(false);
        }
        let id = if method == "elicitation/create" {
            let request: sdk::CreateElicitationRequest = crate::codec::deserialize(raw)?;
            match request.mode.scope() {
                sdk::ElicitationScope::Request(_) => {
                    return Ok(
                        matches!(lifecycle, Lifecycle::Initializing | Lifecycle::Ready)
                            && request_scope,
                    );
                }
                sdk::ElicitationScope::Session(scope) => scope.session_id.0.to_string(),
                _ => return Ok(false),
            }
        } else {
            sessions::id(raw)?
        };
        if lifecycle != Lifecycle::Ready {
            return Ok(false);
        }
        if !self.sessions.contains_key(&id) && !self.creating.is_empty() {
            if self.sessions.len() >= self.maximum_sessions {
                return Err(Fault::LimitExceeded("Sessions", self.maximum_sessions));
            }
            self.sessions
                .insert(id.clone(), Session::new(&id, &self.budget)?);
        }
        Ok(self.has_session(&id))
    }

    pub fn cancel_session(&self, raw: &RawJson) -> Result<String, Fault> {
        let id = sessions::id(raw)?;
        self.sessions
            .get(&id)
            .filter(|session| session.ready || (session.observed && session.observed_cancel))
            .ok_or(Fault::InvalidArgument("SessionId"))?;
        Ok(id)
    }
    pub fn mark_cancelled(&mut self, id: &str) {
        if let Some(session) = self.sessions.get_mut(id) {
            session.cancelled = session.observed
                || session.prompt.is_some()
                || self.negotiated == Some(Version::V2);
        }
    }
    pub fn close(&mut self) {
        self.sessions.clear();
        self.creating.clear();
    }
    pub fn cancelled(&self, id: &str) -> bool {
        self.sessions
            .get(id)
            .is_some_and(|session| session.cancelled)
    }
}
