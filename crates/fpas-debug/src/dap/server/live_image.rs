//! DAP custom-request mapping for live-image compatibility classification.

use serde_json::{Value, json};

use super::DapServer;

impl DapServer {
    pub(super) fn classify_live_image(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, "reload.classify", json!({}))
    }

    pub(super) fn replace_current_live_image(
        &mut self,
        request_seq: u64,
        command: &str,
    ) -> Vec<Value> {
        self.core_request(request_seq, command, "image.replace", json!({}))
    }

    /// Reject an incompatible candidate before the live image can change.
    ///
    /// Protocol `fpas/reload` gates the current executable. Tests inject a
    /// second compiled image here.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns a session error when the adapter is not inspectable or the
    /// candidate is incompatible.
    pub fn replace_live_image(
        &mut self,
        candidate: &fpas_bytecode::VerifiedExecutable,
    ) -> Result<fpas_vm::LiveImageReplaceResult, fpas_vm::DebugSessionError> {
        self.core.replace_live_image(candidate)
    }
}

/// Translate one live-image custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    if command != "fpas/reloadClassify" && command != "fpas/reload" {
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
