//! Durable data-location identities.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, session_error};
use super::{DebugEngine, DebugStatus};

impl DebugEngine {
    pub(super) fn describe_location(
        &mut self,
        request_id: u64,
        command: &str,
        variables_reference: u64,
        name: String,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.describe_data_location(variables_reference, &name) {
            Ok(location) => vec![ok(request_id, command, ResponseBody::Location(location))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
