//! DAP custom-request mapping for live-image compatibility classification.

use serde_json::{Value, json};

use super::DapServer;
use crate::engine::{DebugOp, ResponseBody};

impl DapServer {
    pub(super) fn classify_live_image(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        self.core_request(request_seq, command, DebugOp::ReloadClassify)
    }

    pub(super) fn replace_current_live_image(
        &mut self,
        request_seq: u64,
        command: &str,
    ) -> Vec<Value> {
        let records = self.core_request(request_seq, command, DebugOp::ImageReplace);
        self.with_reload_invalidation(records)
    }

    pub(super) fn rollback_live_image(&mut self, request_seq: u64, command: &str) -> Vec<Value> {
        let records = self.core_request(request_seq, command, DebugOp::ImageRollback);
        self.with_reload_invalidation(records)
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

    fn with_reload_invalidation(&mut self, mut records: Vec<Value>) -> Vec<Value> {
        let applied = records.iter().any(|record| {
            record.get("type").and_then(Value::as_str) == Some("response")
                && record.get("success").and_then(Value::as_bool) == Some(true)
                && record.pointer("/body/applied").and_then(Value::as_bool) == Some(true)
        });
        if applied && self.supports_invalidated_event {
            records.push(self.event("invalidated", json!({"areas":["stacks","variables"]})));
        }
        records
    }
}

/// Translate one live-image custom-request result into DAP naming.
pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    if !matches!(
        command,
        "fpas/reloadClassify" | "fpas/reload" | "fpas/reloadRollback"
    ) {
        return None;
    }
    let ResponseBody::LiveImage {
        class,
        accepted,
        applied,
        version,
        rollback_available,
    } = body
    else {
        return None;
    };
    Some(json!({
        "class": class.as_str(),
        "accepted": accepted,
        "applied": applied,
        "version": version,
        "rollbackAvailable": rollback_available,
        "acceptedClasses": fpas_vm::LiveImageUpdateClass::ACCEPTED
            .iter()
            .map(|class| class.as_str())
            .collect::<Vec<_>>(),
        "rejectedClasses": fpas_vm::LiveImageUpdateClass::REJECTED
            .iter()
            .map(|class| class.as_str())
            .collect::<Vec<_>>(),
    }))
}
