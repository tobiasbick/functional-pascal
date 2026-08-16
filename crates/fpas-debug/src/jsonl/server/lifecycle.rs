//! JSONL mapping for rejected debugger task create and restart.

use serde_json::{Map, Value};

use super::{JsonlServer, ServerStatus};
use crate::jsonl::encode::{invalid_state, missing_argument};
use crate::jsonl::protocol::session_error;

impl JsonlServer {
    /// Reject debugger-created tasks with a stable capability error.
    pub(super) fn create_task(&mut self, request_id: u64, command: &str) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        vec![session_error(request_id, command, session.create_task())]
    }

    /// Reject task restart after validating an optional current `task_id`.
    pub(super) fn restart_task(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != ServerStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let task_id = match arguments.get("task_id") {
            None | Some(Value::Null) => None,
            Some(value) => match value.as_u64() {
                Some(task_id) => Some(task_id),
                None => {
                    return vec![missing_argument(request_id, command, "task_id")];
                }
            },
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        vec![session_error(
            request_id,
            command,
            session.restart_task(task_id),
        )]
    }
}
