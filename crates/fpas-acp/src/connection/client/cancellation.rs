//! Session cancellation and closure consume the same responder as application replies.

use super::super::{
    Event, Fault, Payload, Responder,
    shared::{Shared, State},
};

pub(in super::super) fn select(state: &State, session: &str) -> Vec<Responder> {
    state
        .responders
        .iter()
        .filter(|(_, entry)| {
            !entry.queued
                && entry.sdk.method() == "session/request_permission"
                && entry.session.as_deref() == Some(session)
        })
        .map(|(token, _)| *token)
        .collect()
}
pub(in super::super) fn select_closed(
    state: &State,
    sessions: &std::collections::BTreeSet<String>,
) -> Vec<Responder> {
    state
        .responders
        .iter()
        .filter(|(_, entry)| {
            !entry.queued
                && entry
                    .session
                    .as_ref()
                    .is_some_and(|id| sessions.contains(id))
        })
        .map(|(token, _)| *token)
        .collect()
}
pub(in super::super) fn mark(state: &mut State, tokens: &[Responder]) {
    for token in tokens {
        if let Some(entry) = state.responders.get_mut(token) {
            entry.queued = true;
        }
    }
}

pub(crate) fn settle(shared: &Shared, responders: &[Responder]) -> Result<(), Fault> {
    for token in responders {
        let entry = shared.lock().responders.remove(token);
        if let Some(entry) = entry {
            let id = entry.sdk.id().clone();
            // Keep the incoming reservation until the SDK response reaches the transport.
            shared.lock().cancelled_replies.insert(id, entry._metadata);
            if entry.sdk.method() == "session/request_permission" {
                entry
                    .sdk
                    .respond_with_result(Ok(serde_json::json!({"outcome":{"outcome":"cancelled"}})))
                    .map_err(|_| Fault::Host)?;
            } else {
                entry
                    .sdk
                    .respond_with_error(agent_client_protocol::Error::request_cancelled())
                    .map_err(|_| Fault::Host)?;
            }
            shared.event(Event {
                payload: Payload::RequestCancelled(*token),
                raw: None,
            })?;
        }
    }
    Ok(())
}
