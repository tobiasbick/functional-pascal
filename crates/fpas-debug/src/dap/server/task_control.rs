//! DAP custom-request mapping for per-task pause and resume holds.

use serde_json::{Value, json};

use super::DapServer;

/// Translate one per-task control response into DAP naming.
pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    match command {
        "fpas/pauseTask" | "fpas/resumeTask" => Some(json!({
            "taskId": body.get("task_id"),
            "paused": body.get("paused")
        })),
        "fpas/cancelTask" => Some(json!({
            "taskId": body.get("task_id"),
            "state": body.get("state")
        })),
        _ => None,
    }
}

impl DapServer {
    pub(super) fn pause_task(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.task_hold_request(request_seq, command, "task.pause", arguments)
    }

    pub(super) fn resume_task(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.task_hold_request(request_seq, command, "task.resume", arguments)
    }

    pub(super) fn cancel_task(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.task_hold_request(request_seq, command, "task.cancel", arguments)
    }

    fn task_hold_request(
        &mut self,
        request_seq: u64,
        command: &str,
        core_command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        match self.task_id(arguments, "threadId") {
            Ok(task_id) => self.core_request(
                request_seq,
                command,
                core_command,
                json!({"task_id": task_id}),
            ),
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }
}
