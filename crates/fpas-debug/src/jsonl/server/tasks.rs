//! JSONL task catalog requests.

use serde_json::{Map, Value, json};

use super::{JsonlServer, ServerStatus};
use crate::jsonl::encode::{index_argument, invalid_state, task_body};
use crate::jsonl::protocol::{session_error, success};

impl JsonlServer {
    /// Returns a page of tasks from the stopped debug session.
    pub(super) fn tasks(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let start = index_argument(arguments, "start", 0);
        let count = index_argument(arguments, "count", 64);
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.tasks(start, count) {
            Ok(tasks) => vec![success(
                request_id,
                command,
                json!({
                    "tasks": tasks.items.iter().map(task_body).collect::<Vec<_>>(),
                    "total": tasks.total
                }),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
