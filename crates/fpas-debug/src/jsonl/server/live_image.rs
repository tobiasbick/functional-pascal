//! JSONL mapping for live-image classification and reject-before-commit.

use serde_json::{Value, json};

use super::{JsonlServer, ServerStatus};
use crate::jsonl::encode::invalid_state;
use crate::jsonl::protocol::{session_error, success};

impl JsonlServer {
    pub(super) fn classify_live_image(&mut self, request_id: u64, command: &str) -> Vec<Value> {
        if !matches!(
            self.status,
            ServerStatus::Initialized | ServerStatus::Stopped
        ) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        let classification = session.classify_current_live_image();
        vec![success(
            request_id,
            command,
            classification_body(classification.class, classification.accepted, false),
        )]
    }

    pub(super) fn replace_current_live_image(
        &mut self,
        request_id: u64,
        command: &str,
    ) -> Vec<Value> {
        if !matches!(
            self.status,
            ServerStatus::Initialized | ServerStatus::Stopped
        ) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.replace_current_live_image() {
            Ok(result) => vec![success(
                request_id,
                command,
                classification_body(result.class, result.accepted, result.applied),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    /// Reject an incompatible candidate before the live image can change.
    ///
    /// Protocol `reload` / `image.replace` gate the current executable.
    /// Tests and later CLI reload pass a second compiled image here.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns a session error when the actor is not ready or the candidate is
    /// incompatible.
    pub fn replace_live_image(
        &mut self,
        candidate: &fpas_bytecode::VerifiedExecutable,
    ) -> Result<fpas_vm::LiveImageReplaceResult, fpas_vm::DebugSessionError> {
        let Some(session) = self.actor.session_mut() else {
            return Err(fpas_vm::DebugSessionError {
                kind: fpas_vm::DebugErrorKind::InvalidState,
                message: "live-image replace is invalid while the debug session is not inspectable"
                    .to_string(),
                hint: "Wait for initialize or a stopped event before replacing the live image."
                    .to_string(),
            });
        };
        session.replace_live_image(candidate)
    }
}

fn classification_body(
    class: fpas_vm::LiveImageUpdateClass,
    accepted: bool,
    applied: bool,
) -> Value {
    json!({
        "class": class.as_str(),
        "accepted": accepted,
        "applied": applied,
        "accepted_classes": fpas_vm::LiveImageUpdateClass::ACCEPTED
            .iter()
            .map(|class| class.as_str())
            .collect::<Vec<_>>(),
        "rejected_classes": fpas_vm::LiveImageUpdateClass::REJECTED
            .iter()
            .map(|class| class.as_str())
            .collect::<Vec<_>>(),
    })
}
