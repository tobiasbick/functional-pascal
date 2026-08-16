//! DAP mapping for rejected live debuggee input.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn push_debuggee_input(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, "io.input", json!({}))
    }
}
