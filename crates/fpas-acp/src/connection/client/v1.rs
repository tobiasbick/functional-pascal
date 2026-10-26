use super::super::{
    Fault, Lifecycle, Limits, RequestToken,
    budget::{Budget, Permit},
};
use super::settings::Settings;
use super::{
    capabilities::{self, require},
    sessions::{self, Session, Sessions},
};
use crate::{RawJson, protocol::v1 as sdk, schema::Version};
use std::{collections::BTreeSet, sync::Arc};

pub(in super::super) enum Action {
    None,
    Initialize,
    InitializeV2,
    New,
    Setup(String, Option<Box<Session>>),
    Prompt(String),
    V2Prompt(String),
    Mcp(u64),
    Close(String),
    Session(String, RequestToken),
}

pub(in super::super) struct Client {
    pub settings: Settings,
    pub initialization: Option<RawJson>,
    pub(super) peer: Option<sdk::InitializeResponse>,
    pub(super) peer_v2: Option<crate::protocol::v2::InitializeResponse>,
    pub(super) negotiated: Option<Version>,
    pub(super) started: bool,
    pub(super) sessions: Sessions,
    pub(super) creating: BTreeSet<RequestToken>,
    pub(super) budget: Arc<Budget>,
    pub(super) maximum_sessions: usize,
    _settings: Permit,
    _v2_settings: Permit,
    pub(super) _peer: Option<Permit>,
}

impl Client {
    pub fn has_session(&self, id: &str) -> bool {
        self.sessions
            .get(id)
            .is_some_and(|session| session.closing.is_none())
    }
    pub fn new(settings: Settings, limits: &Limits) -> Result<Self, Fault> {
        settings.requested_version()?;
        let budget = Budget::new("ClientStateBytes", limits.queue_bytes);
        // Empty SDK capabilities contain no dynamic state. The fixed structs are
        // already covered by the bounded connection count, even for tiny queues.
        let bytes = if settings.client_capabilities == sdk::ClientCapabilities::default() {
            0
        } else {
            serde_json::to_string(&settings.client_capabilities)
                .map_err(|_| Fault::InvalidMessage)?
                .len()
        } + if settings.client_info.is_some() {
            serde_json::to_string(&settings.client_info)
                .map_err(|_| Fault::InvalidMessage)?
                .len()
        } else {
            0
        };
        let credit = budget.reserve(bytes.saturating_mul(16))?;
        let v2_bytes = serde_json::to_string(&settings.v2_client_capabilities)
            .map_err(|_| Fault::InvalidMessage)?
            .len();
        let v2_credit =
            budget.reserve(if settings.v2_client_capabilities == Default::default() {
                0
            } else {
                v2_bytes.saturating_mul(16)
            })?;
        Ok(Self {
            settings,
            initialization: None,
            peer: None,
            peer_v2: None,
            negotiated: None,
            started: false,
            sessions: Sessions::new(),
            creating: BTreeSet::new(),
            budget,
            maximum_sessions: limits.sessions,
            _settings: credit,
            _v2_settings: v2_credit,
            _peer: None,
        })
    }

    pub fn prepare(
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
            if request.protocol_version.as_u16() != 1 {
                return Err(Fault::UnsupportedVersion(request.protocol_version.as_u16()));
            }
            if self.settings.requested_version()? != Version::V1
                || request.client_capabilities != self.settings.client_capabilities
                || request.client_info != self.settings.client_info
            {
                return Err(Fault::InvalidArgument("InitializationOffer"));
            }
            return Ok(Action::Initialize);
        }
        if lifecycle != Lifecycle::Ready {
            return Err(Fault::Busy("Initialize"));
        }
        let peer = self.peer.as_ref().ok_or(Fault::Busy("Initialize"))?;
        let caps = &peer.agent_capabilities;
        self.experimental_fields(raw)?;
        if let Some(action) = self.experimental_request(method, raw, limits)? {
            return Ok(action);
        }
        match method {
            "authenticate" => {
                let request: sdk::AuthenticateRequest = crate::codec::deserialize(raw)?;
                let method = peer
                    .auth_methods
                    .iter()
                    .find(|method| method.id() == &request.method_id)
                    .ok_or(Fault::InvalidArgument("AuthMethodId"))?;
                require(
                    matches!(method, sdk::AuthMethod::Agent(_)),
                    "TerminalAuthentication",
                )?;
                return Ok(Action::None);
            }
            "logout" => {
                require(caps.auth.logout.is_some(), "Logout")?;
                return Ok(Action::None);
            }
            "session/list" => {
                require(caps.session_capabilities.list.is_some(), "SessionList")?;
                return Ok(Action::None);
            }
            "session/delete" => {
                self.reject_observed(raw)?;
                require(caps.session_capabilities.delete.is_some(), "SessionDelete")?;
                return Ok(Action::None);
            }
            "session/new" | "session/load" | "session/resume" => {
                match method {
                    "session/load" => require(caps.load_session, "SessionLoad")?,
                    "session/resume" => {
                        require(caps.session_capabilities.resume.is_some(), "SessionResume")?
                    }
                    _ => {}
                }
                capabilities::setup(caps, raw)?;
                let known = self
                    .sessions
                    .values()
                    .filter(|session| session.ready || session.setup.is_some() || session.observed)
                    .count();
                let provisional = self.sessions.len() - known;
                if known + self.creating.len().max(provisional) >= limits.sessions {
                    return Err(Fault::LimitExceeded("Sessions", limits.sessions));
                }
                if method == "session/new" {
                    return Ok(Action::New);
                }
                let id = sessions::id(raw)?;
                if self.sessions.contains_key(&id) {
                    return Err(Fault::Busy("SessionSetup"));
                }
                let session = Session::new(&id, &self.budget)?;
                return Ok(Action::Setup(id, Some(Box::new(session))));
            }
            _ => {}
        }
        if !matches!(
            method,
            "session/prompt" | "session/set_mode" | "session/set_config_option" | "session/close"
        ) {
            return Err(Fault::UnsupportedCapability("ExperimentalMethod"));
        }
        let id = sessions::id(raw)?;
        let session = self
            .sessions
            .get(&id)
            .filter(|session| session.ready)
            .ok_or(Fault::InvalidArgument("SessionId"))?;
        if session.closing.is_some() {
            return Err(Fault::Busy("SessionClose"));
        }
        match method {
            "session/prompt" => {
                if session.prompt.is_some() {
                    return Err(Fault::Busy("SessionPrompt"));
                }
                capabilities::prompt(&caps.prompt_capabilities, raw)?;
                Ok(Action::Prompt(id))
            }
            "session/set_mode" => {
                session.mode(raw)?;
                Ok(Action::Session(
                    id,
                    session.generation.ok_or(Fault::InvalidMessage)?,
                ))
            }
            "session/set_config_option" => {
                session.config(raw)?;
                Ok(Action::Session(
                    id,
                    session.generation.ok_or(Fault::InvalidMessage)?,
                ))
            }
            "session/close" => {
                require(caps.session_capabilities.close.is_some(), "SessionClose")?;
                Ok(Action::Close(id))
            }
            _ => Err(Fault::UnsupportedCapability("ExperimentalMethod")),
        }
    }

    #[expect(
        clippy::expect_used,
        reason = "Prepared actions are admitted once under the same state lock as their session validation"
    )]
    pub fn admit(&mut self, action: &mut Action, token: RequestToken) {
        match action {
            Action::Initialize | Action::InitializeV2 => self.started = true,
            Action::New => {
                self.creating.insert(token);
            }
            Action::Setup(id, reserved) => {
                let mut session = reserved
                    .take()
                    .expect("setup admission owns its reservation");
                session.setup = Some(token);
                session.generation = Some(token);
                self.sessions.insert(id.clone(), *session);
            }
            Action::Prompt(id) => {
                let session = self
                    .sessions
                    .get_mut(id)
                    .expect("validated session remains under admission lock");
                session.prompt = Some(token);
                session.cancelled = false;
            }
            Action::V2Prompt(id) => {
                if let Some(session) = self.sessions.get_mut(id) {
                    session.cancelled = false;
                }
            }
            Action::Close(id) => {
                if let Some(session) = self.sessions.get_mut(id) {
                    session.closing = Some(token);
                }
            }
            _ => {}
        }
    }

    pub fn settle(
        &mut self,
        action: &Action,
        token: RequestToken,
        response: Option<&RawJson>,
    ) -> Result<(), Fault> {
        match action {
            Action::Initialize => {
                if let Some(raw) = response {
                    let peer: sdk::InitializeResponse = crate::codec::deserialize(raw)?;
                    if peer.protocol_version.as_u16() != 1 {
                        return Err(Fault::UnsupportedVersion(peer.protocol_version.as_u16()));
                    }
                    let credit = self.budget.reserve(sessions::retained_size(raw))?;
                    self.initialization = Some(raw.clone());
                    self.peer = Some(peer);
                    self.negotiated = Some(Version::V1);
                    self._peer = Some(credit);
                }
            }
            Action::New => {
                self.creating.remove(&token);
                if let Some(raw) = response {
                    let id = sessions::id(raw)?;
                    if !self.sessions.contains_key(&id) {
                        if self.creating.is_empty() {
                            self.sessions.retain(|_, session| {
                                session.ready || session.setup.is_some() || session.observed
                            });
                        }
                        if self.sessions.len() >= self.maximum_sessions {
                            return Err(Fault::LimitExceeded("Sessions", self.maximum_sessions));
                        }
                        self.sessions
                            .insert(id.clone(), Session::new(&id, &self.budget)?);
                    }
                    let session = self.sessions.get_mut(&id).ok_or(Fault::InvalidMessage)?;
                    if session.ready || session.setup.is_some() || session.observed {
                        return Err(Fault::InvalidMessage);
                    }
                    session.replace_version(
                        raw,
                        &self.budget,
                        self.negotiated.unwrap_or(Version::V1),
                    )?;
                    session.ready = true;
                    session.generation = Some(token);
                }
                if self.creating.is_empty() {
                    self.sessions.retain(|_, session| {
                        session.ready || session.setup.is_some() || session.observed
                    });
                }
            }
            Action::Setup(id, _) => {
                if let Some(raw) = response {
                    let session = self.sessions.get_mut(id).ok_or(Fault::InvalidMessage)?;
                    session.replace_version(
                        raw,
                        &self.budget,
                        self.negotiated.unwrap_or(Version::V1),
                    )?;
                    session.ready = true;
                    session.setup = None;
                } else {
                    self.sessions.remove(id);
                }
            }
            Action::Prompt(id) => {
                if let Some(session) = self.sessions.get_mut(id)
                    && session.prompt == Some(token)
                {
                    session.prompt = None;
                    session.cancelled = false;
                }
            }
            Action::Close(id) => {
                if response.is_some() {
                    let closed = self.session_tree(id);
                    self.sessions.retain(|id, _| !closed.contains(id));
                } else if let Some(session) = self.sessions.get_mut(id)
                    && session.closing == Some(token)
                {
                    session.closing = None;
                }
            }
            Action::Session(id, generation) => {
                if let Some(raw) = response
                    && let Some(session) = self.sessions.get_mut(id)
                    && session.generation == Some(*generation)
                {
                    session.replace_version(
                        raw,
                        &self.budget,
                        self.negotiated.unwrap_or(Version::V1),
                    )?;
                }
            }
            Action::None => {}
            Action::Mcp(_) => {}
            Action::V2Prompt(_) => {}
            Action::InitializeV2 => self.settle_v2_initialization(response)?,
        }
        Ok(())
    }
}
