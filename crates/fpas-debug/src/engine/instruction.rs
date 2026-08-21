//! Rejected instruction-pointer changes.

use super::record::DebugRecord;
use super::reply::{invalid_state, session_error};
use super::{DebugEngine, DebugStatus};

impl DebugEngine {
    pub(super) fn set_instruction(
        &mut self,
        request_id: u64,
        command: &str,
        frame_id: Option<u64>,
        instruction: Option<u32>,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        vec![session_error(
            request_id,
            command,
            session.set_instruction(frame_id, instruction),
        )]
    }
}
