//! JSONL mapping for per-task pause and resume holds.

use serde_json::{Map, Value, json};

use super::{DebugEngine, DebugStatus};
use crate::jsonl::encode::{invalid_state, required_u64_argument, task_event};
use crate::jsonl::protocol::{session_error, success};

impl DebugEngine {
    /// Hold one current runtime task so later continue and peer steps skip it.
    pub(super) fn pause_task(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        self.set_task_hold(request_id, command, arguments, true)
    }

    /// Clear the hold on one current runtime task without resuming the session.
    pub(super) fn resume_task(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        self.set_task_hold(request_id, command, arguments, false)
    }

    fn set_task_hold(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
        paused: bool,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let task_id = match required_u64_argument(request_id, command, arguments, "task_id") {
            Ok(task_id) => task_id,
            Err(error) => return vec![error],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        let result = if paused {
            session.pause_task(task_id)
        } else {
            session.resume_task(task_id)
        };
        match result {
            Ok(()) => vec![success(
                request_id,
                command,
                json!({"task_id": task_id, "paused": paused}),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    /// Cancel one live non-root runtime task at the current stop.
    pub(super) fn cancel_task(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let task_id = match required_u64_argument(request_id, command, arguments, "task_id") {
            Ok(task_id) => task_id,
            Err(error) => return vec![error],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.cancel_task(task_id) {
            Ok(()) => {
                let mut records = vec![success(
                    request_id,
                    command,
                    json!({"task_id": task_id, "state": "cancelled"}),
                )];
                records.extend(session.take_task_events().into_iter().map(task_event));
                records
            }
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
