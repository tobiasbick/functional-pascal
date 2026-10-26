//! V2 requests accept prompts independently of later foreground state updates.
use super::super::{Fault, Lifecycle, Limits};
use super::{
    capabilities::require,
    experimental::object,
    sessions::{self, Session},
    v1::{Action, Client},
};
use crate::{RawJson, protocol::v2 as sdk, schema::Version};

impl Client {
    pub fn negotiated_version(&self) -> Option<Version> {
        self.negotiated
    }
    pub fn prepare_v2(
        &self,
        lifecycle: Lifecycle,
        method: &str,
        raw: &RawJson,
        limits: &Limits,
    ) -> Result<Action, Fault> {
        if method.starts_with('_') {
            return Ok(Action::None);
        }
        if method == "initialize" {
            if self.started {
                return Err(Fault::Busy("Initialize"));
            }
            let request: sdk::InitializeRequest = crate::codec::deserialize(raw)?;
            if request.protocol_version.as_u16() != 2 {
                return Err(Fault::UnsupportedVersion(request.protocol_version.as_u16()));
            }
            if self.settings.requested_version()? != Version::V2
                || serde_json::to_value(&self.settings.client_info)
                    .map_err(|_| Fault::InvalidMessage)?
                    != serde_json::to_value(&request.info).map_err(|_| Fault::InvalidMessage)?
                || self.settings.v2_client_capabilities != request.capabilities
            {
                return Err(Fault::InvalidArgument("InitializationOffer"));
            }
            return Ok(Action::InitializeV2);
        }
        if lifecycle != Lifecycle::Ready {
            return Err(Fault::Busy("Initialize"));
        }
        self.experimental_fields(raw)?;
        if let Some(action) = self.experimental_request(method, raw, limits)? {
            return Ok(action);
        }
        let peer = self.peer_v2.as_ref().ok_or(Fault::Busy("Initialize"))?;
        let caps = self.peer_capabilities()?;
        match method {
            "auth/login" => {
                let request: sdk::LoginAuthRequest = crate::codec::deserialize(raw)?;
                require(!peer.auth_methods.is_empty(), "Authentication")?;
                let method = peer
                    .auth_methods
                    .iter()
                    .find(|method| method.method_id() == &request.method_id)
                    .ok_or(Fault::InvalidArgument("AuthMethodId"))?;
                require(
                    matches!(method, sdk::AuthMethod::Agent(_)),
                    "TerminalAuthentication",
                )?;
                return Ok(Action::None);
            }
            "auth/logout" => {
                require(!peer.auth_methods.is_empty(), "Authentication")?;
                return Ok(Action::None);
            }
            "session/list" => return Ok(Action::None),
            "session/delete" => {
                self.reject_observed(raw)?;
                require(object(&caps["session"]["delete"]), "SessionDelete")?;
                return Ok(Action::None);
            }
            "session/new" | "session/resume" => {
                self.setup_v2(raw)?;
                if self.sessions.len() + self.creating.len() >= limits.sessions {
                    return Err(Fault::LimitExceeded("Sessions", limits.sessions));
                }
                if method == "session/new" {
                    return Ok(Action::New);
                }
                let id = sessions::id(raw)?;
                if self.sessions.contains_key(&id) {
                    return Err(Fault::Busy("SessionSetup"));
                }
                return Ok(Action::Setup(
                    id.clone(),
                    Some(Box::new(Session::new(&id, &self.budget)?)),
                ));
            }
            _ => {}
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
        match method {
            "session/prompt" => {
                let prompt: sdk::PromptRequest = crate::codec::deserialize(raw)?;
                for content in prompt.prompt {
                    match content {
                        sdk::ContentBlock::Image(_) => {
                            require(object(&caps["session"]["prompt"]["image"]), "PromptImage")?
                        }
                        sdk::ContentBlock::Audio(_) => {
                            require(object(&caps["session"]["prompt"]["audio"]), "PromptAudio")?
                        }
                        sdk::ContentBlock::Resource(_) => require(
                            object(&caps["session"]["prompt"]["embeddedContext"]),
                            "PromptEmbeddedContext",
                        )?,
                        _ => {}
                    }
                }
                Ok(Action::V2Prompt(id))
            }
            "session/set_config_option" => {
                session.config_v2(raw)?;
                Ok(Action::Session(
                    id,
                    session.generation.ok_or(Fault::InvalidMessage)?,
                ))
            }
            "session/close" => Ok(Action::Close(id)),
            _ => Err(Fault::UnsupportedCapability("V2Method")),
        }
    }
    pub fn setup_v2(&self, raw: &RawJson) -> Result<(), Fault> {
        let caps = self.peer_capabilities()?;
        if let Some(roots) = raw.member("additionalDirectories")?
            && roots.text() != "null"
            && roots.text() != "[]"
        {
            require(
                object(&caps["session"]["additionalDirectories"]),
                "AdditionalDirectories",
            )?;
        }
        if let Some(servers) = raw.member("mcpServers")? {
            if servers.text() == "null" {
                return Ok(());
            }
            let servers: Vec<sdk::McpServer> = crate::codec::deserialize(&servers)?;
            for server in servers {
                match server {
                    sdk::McpServer::Stdio(_) => {
                        require(object(&caps["session"]["mcp"]["stdio"]), "McpStdio")?
                    }
                    sdk::McpServer::Http(_) => {
                        require(object(&caps["session"]["mcp"]["http"]), "McpHttp")?
                    }
                    sdk::McpServer::Acp(_) => self.require_mcp()?,
                    _ => return Err(Fault::UnsupportedCapability("McpTransport")),
                }
            }
        }
        Ok(())
    }
    pub fn settle_v2_initialization(&mut self, response: Option<&RawJson>) -> Result<(), Fault> {
        if let Some(raw) = response {
            let version: u16 = crate::codec::deserialize(
                &raw.member("protocolVersion")?
                    .ok_or(Fault::InvalidMessage)?,
            )?;
            let credit = self.budget.reserve(sessions::retained_size(raw))?;
            match version {
                2 => {
                    self.peer_v2 = Some(crate::codec::deserialize(raw)?);
                    self.negotiated = Some(Version::V2);
                }
                1 if self.settings.supported_versions.contains(&Version::V1) => {
                    self.peer = Some(crate::codec::deserialize(raw)?);
                    self.negotiated = Some(Version::V1);
                    // A v1 agent did not receive v1 clientCapabilities in the v2 offer.
                    self.settings.client_capabilities = Default::default();
                }
                _ => return Err(Fault::UnsupportedVersion(version)),
            }
            self.initialization = Some(raw.clone());
            self._peer = Some(credit);
        }
        Ok(())
    }
    pub fn allow_incoming_v2(
        &mut self,
        lifecycle: Lifecycle,
        method: &str,
        raw: &RawJson,
        request_scope: bool,
    ) -> Result<bool, Fault> {
        if method.starts_with('_') {
            return Ok(true);
        }
        if method == "elicitation/create" {
            let request: sdk::CreateElicitationRequest = crate::codec::deserialize(raw)?;
            let Some(caps) = &self.settings.v2_client_capabilities.elicitation else {
                return Ok(false);
            };
            let enabled = match &request.mode {
                sdk::ElicitationMode::Form(_) => caps.form.is_some(),
                sdk::ElicitationMode::Url(_) => caps.url.is_some(),
                _ => false,
            };
            if !enabled {
                return Ok(false);
            }
            return Ok(match request.mode.scope() {
                sdk::ElicitationScope::Request(_) => request_scope,
                sdk::ElicitationScope::Session(scope) => {
                    lifecycle == Lifecycle::Ready && self.has_session(scope.session_id.0.as_ref())
                }
                _ => false,
            });
        }
        if lifecycle != Lifecycle::Ready {
            return Ok(false);
        }
        if method == "mcp/message" {
            return Ok(self.settings.experimental.mcp);
        }
        if method == "session/request_permission" {
            return Ok(self.has_session(&sessions::id(raw)?));
        }
        Ok(false)
    }
}
