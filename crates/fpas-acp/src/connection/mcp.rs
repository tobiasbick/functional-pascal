//! Explicit MCP endpoints retain scoped carrier routing until removal or close.
use super::{
    Connection, Fault, Lifecycle,
    budget::Permit,
    shared::{Command, SERIAL_MASK, Shared, State},
};
use crate::RawJson;
const TAG: u64 = 0x4147_0000_0000_0000;
pub(super) struct Endpoint {
    pub id: u64,
    pub session: Option<String>,
    _credit: Permit,
}

pub(super) fn target(raw: &RawJson) -> Result<String, Fault> {
    let id = raw
        .member("serverId")?
        .ok_or(Fault::InvalidArgument("McpServerId"))?;
    crate::codec::deserialize(&id).map_err(Into::into)
}
pub(super) fn route(state: &State, raw: &RawJson, setup: bool) -> Result<u64, Fault> {
    let id = target(raw)?;
    let endpoint = state
        .routes
        .get(&id)
        .ok_or(Fault::InvalidArgument("McpServerId"))?;
    if let Some(session) = &endpoint.session {
        if setup {
            let requested = raw
                .member("sessionId")?
                .map(|raw| crate::codec::deserialize::<String>(&raw))
                .transpose()?;
            if requested.as_ref() != Some(session) {
                return Err(Fault::InvalidArgument("McpScope"));
            }
        } else if !state.client.has_session(session) {
            return Err(Fault::Closed);
        }
    }
    Ok(endpoint.id)
}
pub(super) fn setup(state: &State, raw: &RawJson, method: &str) -> Result<(), Fault> {
    if let Some(servers) = raw.member("mcpServers")? {
        if servers.text() == "null" {
            return Ok(());
        }
        let servers: Vec<serde_json::Value> =
            serde_json::from_str(servers.text()).map_err(|_| Fault::InvalidMessage)?;
        for server in servers {
            if server["type"] == "acp" {
                let id = server["serverId"]
                    .as_str()
                    .ok_or(Fault::InvalidArgument("McpServerId"))?;
                let endpoint = state
                    .routes
                    .get(id)
                    .ok_or(Fault::InvalidArgument("McpServerId"))?;
                if let Some(scope) = &endpoint.session {
                    if method == "session/fork" {
                        return Err(Fault::InvalidArgument("McpScope"));
                    }
                    let requested = raw
                        .member("sessionId")?
                        .map(|raw| crate::codec::deserialize::<String>(&raw))
                        .transpose()?;
                    if requested.as_ref() != Some(scope) {
                        return Err(Fault::InvalidArgument("McpScope"));
                    }
                }
            }
        }
    }
    Ok(())
}
pub(super) fn retire(shared: &Shared, state: &mut State, key: &str, id: u64) -> Result<(), Fault> {
    let tokens = state.pending.iter().filter(|(_, pending)| matches!(pending.action, super::client::Action::Mcp(endpoint) if endpoint == id)).map(|(token, _)| *token).collect::<Vec<_>>();
    let responders = state
        .responders
        .iter()
        .filter(|(_, entry)| entry.route == Some(id) && !entry.queued)
        .map(|(token, _)| *token)
        .collect::<Vec<_>>();
    let bytes = 256usize
        .saturating_add(tokens.len().saturating_mul(256))
        .saturating_add(responders.len().saturating_mul(16));
    if !shared.commands.push(
        Command::RetireRoute {
            tokens,
            responders: responders.clone(),
        },
        bytes,
    )? {
        return Err(Fault::LimitExceeded(
            "CommandQueue",
            shared.limits.queue_bytes,
        ));
    }
    super::client::cancellation::mark(state, &responders);
    state.routes.remove(key);
    shared.changed.notify_waiters();
    Ok(())
}
impl Connection {
    /// Resolve a live endpoint identity without accepting a foreign connection alias.
    pub fn mcp_endpoint(&self, endpoint: u64) -> Result<(), Fault> {
        let state = self.shared.lock();
        Shared::active(&state)?;
        if state.routes.values().any(|value| value.id == endpoint) {
            Ok(())
        } else {
            Err(Fault::Closed)
        }
    }
    /// Register a unique connection-global server ID, optionally scoped to one session.
    /// The application serves typed carrier events; registering grants no tool authority.
    pub fn register_mcp_endpoint(&self, server: &str, session: Option<&str>) -> Result<u64, Fault> {
        let mut state = self.shared.lock();
        Shared::active(&state)?;
        if state.lifecycle != Lifecycle::Ready {
            return Err(Fault::Busy("Initialize"));
        }
        state.client.require_mcp()?;
        if server.is_empty()
            || server.contains('\0')
            || session.is_some_and(|id| id.is_empty() || id.contains('\0'))
        {
            return Err(Fault::InvalidArgument("McpServerId"));
        }
        if state.routes.contains_key(server) {
            return Err(Fault::Busy("McpEndpoint"));
        }
        if state.routes.len() >= self.shared.limits.routes {
            return Err(Fault::LimitExceeded("Routes", self.shared.limits.routes));
        }
        if state.route_serial == SERIAL_MASK {
            return Err(Fault::LimitExceeded("McpEndpoints", SERIAL_MASK as usize));
        }
        let credit = self.shared.outgoing_metadata.reserve(
            server
                .len()
                .saturating_add(session.map_or(0, str::len))
                .saturating_mul(4)
                .saturating_add(512),
        )?;
        state.route_serial += 1;
        let id = self.shared.identity(TAG, state.route_serial);
        state.routes.insert(
            server.into(),
            Endpoint {
                id,
                session: session.map(str::to_owned),
                _credit: credit,
            },
        );
        Ok(id)
    }
    /// Retire an endpoint, cancel its outstanding carriers and invalidate every alias.
    pub fn remove_mcp_endpoint(&self, endpoint: u64) -> Result<bool, Fault> {
        let mut state = self.shared.lock();
        Shared::active(&state)?;
        let key = state
            .routes
            .iter()
            .find(|(_, value)| value.id == endpoint)
            .map(|(key, _)| key.clone())
            .ok_or(Fault::Closed)?;
        retire(&self.shared, &mut state, &key, endpoint)?;
        Ok(true)
    }
}
