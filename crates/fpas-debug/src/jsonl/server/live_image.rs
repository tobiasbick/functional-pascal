//! JSONL mapping for live-image compatibility classification.

use serde_json::{Value, json};

use super::{JsonlServer, ServerStatus};
use crate::jsonl::encode::invalid_state;
use crate::jsonl::protocol::success;

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
            classification_body(classification),
        )]
    }
}

fn classification_body(classification: fpas_vm::LiveImageClassification) -> Value {
    json!({
        "class": classification.class.as_str(),
        "accepted": classification.accepted,
        "applied": false,
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
