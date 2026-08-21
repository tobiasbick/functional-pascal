//! DAP custom-request mapping for per-task pause and resume holds.

use serde_json::{Value, json};

use super::DapServer;
use crate::engine::{DebugOp, ResponseBody};

/// Translate one per-task control response into DAP naming.
pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    match (command, body) {
        ("fpas/pauseTask" | "fpas/resumeTask", ResponseBody::TaskHold { task_id, paused }) => {
            Some(json!({
                "taskId": task_id,
                "paused": paused
            }))
        }
        ("fpas/cancelTask", ResponseBody::TaskCancelled { task_id }) => Some(json!({
            "taskId": task_id,
            "state": "cancelled"
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
        self.task_hold_request(request_seq, command, arguments, |task_id| {
            DebugOp::TaskPause { task_id }
        })
    }

    pub(super) fn resume_task(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.task_hold_request(request_seq, command, arguments, |task_id| {
            DebugOp::TaskResume { task_id }
        })
    }

    pub(super) fn cancel_task(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        self.task_hold_request(request_seq, command, arguments, |task_id| {
            DebugOp::TaskCancel { task_id }
        })
    }

    fn task_hold_request(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
        op: impl FnOnce(u64) -> DebugOp,
    ) -> Vec<Value> {
        match self.task_id(arguments, "threadId") {
            Ok(task_id) => self.core_request(request_seq, command, op(task_id)),
            Err(message) => vec![self.failure(request_seq, command, &message)],
        }
    }
}
