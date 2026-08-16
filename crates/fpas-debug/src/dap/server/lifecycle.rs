//! DAP mapping for rejected debugger task create and restart.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn create_task(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, "task.create", json!({}))
    }

    pub(super) fn restart_task(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        match arguments.get("threadId") {
            None | Some(Value::Null) => {
                self.core_request(request_seq, command, "task.restart", json!({}))
            }
            Some(_) => match self.task_id(arguments, "threadId") {
                Ok(task_id) => self.core_request(
                    request_seq,
                    command,
                    "task.restart",
                    json!({"task_id": task_id}),
                ),
                Err(message) => vec![self.failure(request_seq, command, &message)],
            },
        }
    }
}
