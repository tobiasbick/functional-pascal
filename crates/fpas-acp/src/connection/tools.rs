//! HTTP tool publications follow admitted setup requests and confirmed session identities.

use super::{Connection, Fault, RequestToken, shared::Shared};
use crate::RawJson;
use fpas_mcp::{CleanupReceipt, HttpOptions, HttpRuntime, Publication, Server};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

#[derive(Clone)]
struct Source {
    identity: Arc<()>,
    name: String,
    server: Server,
    runtime: Arc<HttpRuntime>,
    owned: bool,
}
pub(super) struct Setup {
    sources: Vec<Source>,
    session: Option<String>,
}
struct Attached {
    source: String,
    session: Option<String>,
    publication: Arc<Publication>,
}
impl Drop for Attached {
    fn drop(&mut self) {
        self.publication.stop();
    }
}
#[derive(Default)]
struct State {
    stopping: bool,
    sources: BTreeMap<String, Source>,
    pending: BTreeMap<RequestToken, Vec<Attached>>,
    active: Vec<Attached>,
    receipts: Vec<(String, CleanupReceipt)>,
}
#[derive(Default)]
pub(super) struct Bridge(Mutex<State>);

impl Bridge {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub(super) fn plan(
        &self,
        method: &str,
        raw: &RawJson,
    ) -> Result<(RawJson, Option<Setup>), Fault> {
        if !matches!(
            method,
            "session/new" | "session/load" | "session/resume" | "session/fork"
        ) {
            return Ok((raw.clone(), None));
        }
        let state = self.lock();
        if state.sources.is_empty() {
            return Ok((raw.clone(), None));
        }
        if state.stopping {
            return Err(Fault::Closed);
        }
        let sources = state.sources.values().cloned().collect::<Vec<_>>();
        let session = if matches!(method, "session/load" | "session/resume") {
            Some(
                raw.member("sessionId")?
                    .ok_or(Fault::InvalidMessage)
                    .and_then(|r| crate::codec::deserialize(&r).map_err(Into::into))?,
            )
        } else {
            None
        };
        let mut value: serde_json::Value =
            serde_json::from_str(raw.text()).map_err(|_| Fault::InvalidMessage)?;
        let object = value.as_object_mut().ok_or(Fault::InvalidMessage)?;
        let servers = object
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .ok_or(Fault::InvalidMessage)?;
        for source in &sources {
            if servers
                .iter()
                .any(|s| s.get("name").and_then(serde_json::Value::as_str) == Some(&source.name))
            {
                return Err(Fault::InvalidArgument("McpServerName"));
            }
            servers.push(serde_json::json!({"type":"http","name":source.name,"url":"http://127.0.0.1:65535/mcp","headers":[{"name":"Authorization","value":format!("Bearer {}", "0".repeat(64))}]}));
        }
        Ok((
            RawJson::parse(&value.to_string())?,
            Some(Setup { sources, session }),
        ))
    }
    pub(super) async fn prepare(
        &self,
        shared: &Shared,
        token: RequestToken,
        raw: &RawJson,
        setup: &Setup,
    ) -> Result<RawJson, Fault> {
        let mut value: serde_json::Value =
            serde_json::from_str(raw.text()).map_err(|_| Fault::InvalidMessage)?;
        for source in &setup.sources {
            let mut ticket = {
                let mut state = self.lock();
                if state.stopping
                    || state
                        .sources
                        .get(&source.name)
                        .is_none_or(|current| !Arc::ptr_eq(&current.identity, &source.identity))
                {
                    return Err(Fault::Closed);
                }
                let ticket = source
                    .runtime
                    .publish_attached(source.server.clone(), HttpOptions::default(), shared.id)
                    .map_err(|_| Fault::Host)?;
                state.receipts.retain(|(_, r)| r.result().is_none());
                state
                    .receipts
                    .push((source.name.clone(), ticket.cleanup_receipt()));
                ticket
            };
            let publication = loop {
                if let Some(result) = ticket.try_complete() {
                    break result.map_err(|_| Fault::Host)?;
                }
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            };
            let mut state = self.lock();
            if state.stopping
                || state
                    .sources
                    .get(&source.name)
                    .is_none_or(|current| !Arc::ptr_eq(&current.identity, &source.identity))
            {
                publication.stop();
                return Err(Fault::Closed);
            }
            let descriptor = value["mcpServers"]
                .as_array_mut()
                .ok_or(Fault::InvalidMessage)?
                .iter_mut()
                .find(|v| v["name"].as_str() == Some(&source.name))
                .ok_or(Fault::InvalidMessage)?;
            descriptor["url"] = publication.url().into();
            descriptor["headers"] =
                serde_json::json!([{"name":"Authorization","value":publication.authorization()}]);
            state.pending.entry(token).or_default().push(Attached {
                source: source.name.clone(),
                session: setup.session.clone(),
                publication,
            });
        }
        RawJson::parse(&value.to_string()).map_err(Into::into)
    }
    pub(super) fn settle(
        &self,
        token: RequestToken,
        response: Option<&RawJson>,
    ) -> Result<(), Fault> {
        let mut state = self.lock();
        let Some(mut attached) = state.pending.remove(&token) else {
            return Ok(());
        };
        let Some(response) = response else {
            for a in attached {
                a.publication.stop();
            }
            return Ok(());
        };
        let returned: Option<String> = response
            .member("sessionId")?
            .map(|r| crate::codec::deserialize(&r))
            .transpose()?;
        for item in &mut attached {
            let session = item
                .session
                .as_ref()
                .or(returned.as_ref())
                .ok_or(Fault::InvalidMessage)?;
            if item.publication.bind_session(session).is_err() {
                for item in &attached {
                    item.publication.stop();
                }
                return Err(Fault::Host);
            }
            item.session = Some(session.clone());
        }
        state.active.extend(attached);
        Ok(())
    }
    pub(super) fn retire_sessions(&self, sessions: &BTreeSet<String>) {
        self.lock().active.retain(|a| {
            if a.session.as_ref().is_some_and(|s| sessions.contains(s)) {
                a.publication.stop();
                false
            } else {
                true
            }
        });
    }
    pub(super) fn shutdown(&self) {
        let mut state = self.lock();
        state.stopping = true;
        for a in state.active.drain(..) {
            a.publication.stop();
        }
        for (_, attached) in std::mem::take(&mut state.pending) {
            for a in attached {
                a.publication.stop();
            }
        }
        for source in state.sources.values() {
            if source.owned {
                source.server.close();
            }
        }
        state.sources.clear();
    }
    pub(super) async fn join(&self) -> Result<(), Fault> {
        let receipts = self.lock().receipts.clone();
        let mut result = Ok(());
        for (_, receipt) in receipts {
            if receipt.wait().await.is_err() {
                result = Err(Fault::Host);
            }
        }
        result
    }
}
impl Drop for Bridge {
    fn drop(&mut self) {
        self.shutdown();
    }
}
impl Connection {
    /// Attach a native server before session setup. Ownership controls connection-close behavior.
    pub fn attach_tool_server(
        &self,
        name: &str,
        server: Server,
        runtime: Arc<HttpRuntime>,
        owned: bool,
    ) -> Result<bool, Fault> {
        let admission = self.shared.lock();
        Shared::active(&admission)?;
        if name.is_empty()
            || name.len() > 128
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        {
            return Err(Fault::InvalidArgument("McpServerName"));
        }
        let mut state = self.shared.tools.lock();
        if state.stopping {
            return Err(Fault::Closed);
        }
        if state.sources.contains_key(name) {
            return Ok(false);
        }
        if state.sources.len() == 16 {
            return Err(Fault::LimitExceeded("McpServers", 16));
        }
        state.sources.insert(
            name.into(),
            Source {
                identity: Arc::new(()),
                name: name.into(),
                server,
                runtime,
                owned,
            },
        );
        Ok(true)
    }
    /// Retire this server's attachments while preserving borrowed native server ownership.
    pub fn detach_tool_server(&self, name: &str) -> Result<(bool, Vec<CleanupReceipt>), Fault> {
        let admission = self.shared.lock();
        Shared::active(&admission)?;
        let mut state = self.shared.tools.lock();
        let Some(source) = state.sources.remove(name) else {
            return Ok((false, vec![]));
        };
        let receipts = state
            .receipts
            .iter()
            .filter(|(source, _)| source == name)
            .map(|(_, receipt)| receipt.clone())
            .collect();
        for a in state
            .active
            .iter()
            .chain(state.pending.values().flatten())
            .filter(|a| a.source == name)
        {
            a.publication.stop();
        }
        state.active.retain(|a| a.source != name);
        for pending in state.pending.values_mut() {
            pending.retain(|a| a.source != name);
        }
        if source.owned {
            source.server.close();
        }
        Ok((true, receipts))
    }
}
