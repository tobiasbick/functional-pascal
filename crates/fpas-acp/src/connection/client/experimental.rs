//! Feature opt-ins and peer advertisements are independent admission requirements.
use super::super::{Fault, Limits};
use super::{
    capabilities::require,
    sessions::{self, Session},
    v1::{Action, Client},
};
use crate::{RawJson, schema::Version};
use serde_json::Value;

pub(super) fn object(value: &Value) -> bool {
    value.is_object()
}
impl Client {
    pub fn reject_observed(&self, raw: &RawJson) -> Result<(), Fault> {
        let id = sessions::id(raw)?;
        if self
            .sessions
            .get(&id)
            .is_some_and(|session| session.observed)
        {
            return Err(Fault::InvalidArgument("SessionId"));
        }
        Ok(())
    }
    pub fn peer_capabilities(&self) -> Result<Value, Fault> {
        if self.negotiated == Some(Version::V2) {
            serde_json::to_value(
                &self
                    .peer_v2
                    .as_ref()
                    .ok_or(Fault::Busy("Initialize"))?
                    .capabilities,
            )
        } else {
            serde_json::to_value(
                &self
                    .peer
                    .as_ref()
                    .ok_or(Fault::Busy("Initialize"))?
                    .agent_capabilities,
            )
        }
        .map_err(|_| Fault::InvalidMessage)
    }
    pub fn experimental_fields(&self, raw: &RawJson) -> Result<(), Fault> {
        if let Some(servers) = raw.member("mcpServers")? {
            let servers: Value =
                serde_json::from_str(servers.text()).map_err(|_| Fault::InvalidMessage)?;
            if servers
                .as_array()
                .is_some_and(|servers| servers.iter().any(|server| server["type"] == "acp"))
            {
                self.require_mcp()?;
            }
        }
        if let Some(prompt) = raw.member("systemPrompt")? {
            require(
                self.settings.experimental.system_prompt,
                "SessionSystemPrompt",
            )?;
            let caps = self.peer_capabilities()?;
            let caps = if self.negotiated == Some(Version::V2) {
                &caps["session"]["systemPrompt"]
            } else {
                &caps["sessionCapabilities"]["systemPrompt"]
            };
            require(object(caps), "SessionSystemPrompt")?;
            if prompt.text() != "null" {
                let prompt: Value =
                    serde_json::from_str(prompt.text()).map_err(|_| Fault::InvalidMessage)?;
                match prompt["mode"].as_str().unwrap_or("append") {
                    "append" => {}
                    "override" => {
                        require(object(&caps["mode"]["override"]), "SystemPromptOverride")?
                    }
                    _ => return Err(Fault::UnsupportedCapability("SystemPromptMode")),
                }
            }
        }
        Ok(())
    }
    pub fn experimental_request(
        &self,
        method: &str,
        raw: &RawJson,
        limits: &Limits,
    ) -> Result<Option<Action>, Fault> {
        let flags = self.settings.experimental;
        if method.starts_with("providers/") {
            require(flags.providers, "Providers")?;
            require(object(&self.peer_capabilities()?["providers"]), "Providers")?;
            return Ok(Some(Action::None));
        }
        if method == "mcp/message" {
            self.require_mcp()?;
            return Ok(Some(Action::None));
        }
        if method == "session/fork" {
            require(flags.fork, "SessionFork")?;
            let caps = self.peer_capabilities()?;
            let session_caps = if self.negotiated == Some(Version::V2) {
                &caps["session"]
            } else {
                &caps["sessionCapabilities"]
            };
            require(object(&session_caps["fork"]), "SessionFork")?;
            if self.negotiated == Some(Version::V2) {
                self.setup_v2(raw)?;
            } else {
                super::capabilities::setup(
                    &self
                        .peer
                        .as_ref()
                        .ok_or(Fault::Busy("Initialize"))?
                        .agent_capabilities,
                    raw,
                )?;
            }
            let id = sessions::id(raw)?;
            let session = self
                .sessions
                .get(&id)
                .filter(|session| session.ready && !session.observed)
                .ok_or(Fault::InvalidArgument("SessionId"))?;
            if session.closing.is_some() {
                return Err(Fault::Busy("SessionClose"));
            }
            if self.sessions.len() + self.creating.len() >= limits.sessions {
                return Err(Fault::LimitExceeded("Sessions", limits.sessions));
            }
            return Ok(Some(Action::New));
        }
        Ok(None)
    }
    pub fn require_mcp(&self) -> Result<(), Fault> {
        require(self.settings.experimental.mcp, "McpOverAcp")?;
        let caps = self.peer_capabilities()?;
        let supported = if self.negotiated == Some(Version::V2) {
            object(&caps["session"]["mcp"]["acp"])
        } else {
            caps["mcpCapabilities"]["acp"] == true
        };
        require(supported, "McpOverAcp")
    }
    pub fn experimental_incoming(
        &self,
        method: &str,
        raw: &RawJson,
        version: Version,
    ) -> Result<bool, Fault> {
        if method == "mcp/message" {
            return Ok(self.settings.experimental.mcp);
        }
        if method != "session/update" {
            return Ok(true);
        }
        let update = raw.member("update")?.ok_or(Fault::InvalidMessage)?;
        let kind = update
            .member("sessionUpdate")?
            .ok_or(Fault::InvalidMessage)?;
        let kind: String = crate::codec::deserialize(&kind)?;
        Ok(match kind.as_str() {
            "plan_update" if version == Version::V1 => {
                self.settings.experimental.plans && self.settings.client_capabilities.plan.is_some()
            }
            "plan_removed" => {
                self.settings.experimental.plans
                    && (version == Version::V2 || self.settings.client_capabilities.plan.is_some())
            }
            "subagent_update" | "session_message" | "session_message_chunk" => {
                self.settings.experimental.subagents
                    && (version == Version::V2
                        || self.settings.client_capabilities.subagents.is_some())
            }
            _ => true,
        })
    }
    pub fn observe_subagent(&mut self, parent: &str, update: &RawJson) -> Result<(), Fault> {
        let id = sessions::id(update)?;
        if id == parent {
            return Err(Fault::InvalidMessage);
        }
        if !self.sessions.contains_key(&id) {
            if self.sessions.len() >= self.maximum_sessions {
                return Err(Fault::LimitExceeded("Sessions", self.maximum_sessions));
            }
            let mut session = Session::new(&id, &self.budget)?;
            session.observed = true;
            session.observed_parent = Some((
                parent.into(),
                self.budget.reserve(parent.len().saturating_mul(3))?,
            ));
            self.sessions.insert(id.clone(), session);
        }
        let session = self.sessions.get_mut(&id).ok_or(Fault::InvalidMessage)?;
        if !session.observed
            || session
                .observed_parent
                .as_ref()
                .is_none_or(|(owner, _)| owner != parent)
        {
            return Err(Fault::InvalidMessage);
        }
        // Both families share these child capability/state patch shapes.
        let patch: crate::protocol::v1::SubagentUpdate = crate::codec::deserialize(update)?;
        match patch.capabilities {
            crate::protocol::MaybeUndefined::Undefined => {}
            crate::protocol::MaybeUndefined::Null => session.observed_cancel = false,
            crate::protocol::MaybeUndefined::Value(caps) => {
                session.observed_cancel = caps.cancel.is_some()
            }
        }
        if matches!(
            patch.state,
            crate::protocol::MaybeUndefined::Value(crate::protocol::v1::StateUpdate::Idle(_))
        ) {
            session.cancelled = false;
        }
        Ok(())
    }
}
