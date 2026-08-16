//! JSONL mapping for the session-owned debuggee channel.

use super::{JsonlServer, ServerStatus};
use crate::jsonl::encode::invalid_state;
use crate::jsonl::protocol::session_error;

impl JsonlServer {
    /// Reject live debuggee stdin; protocol stdin is never this channel.
    pub(super) fn push_debuggee_input(
        &mut self,
        request_id: u64,
        command: &str,
    ) -> Vec<serde_json::Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        vec![session_error(
            request_id,
            command,
            session.push_debuggee_input(""),
        )]
    }
}
