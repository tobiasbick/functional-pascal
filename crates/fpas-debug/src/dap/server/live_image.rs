//! DAP custom-request mapping for live-image compatibility classification.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn classify_live_image(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, "reload.classify", json!({}))
    }
}

/// Translate one live-image custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    if command != "fpas/reloadClassify" {
        return None;
    }
    Some(json!({
        "class": body.get("class"),
        "accepted": body.get("accepted"),
        "applied": body.get("applied"),
        "acceptedClasses": body.get("accepted_classes"),
        "rejectedClasses": body.get("rejected_classes"),
    }))
}
