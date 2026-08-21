//! Selected live-frame restart.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, session_error};
use super::{DebugEngine, DebugStatus};

impl DebugEngine {
    pub(super) fn restart_frame(
        &mut self,
        request_id: u64,
        command: &str,
        frame_id: u64,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.restart_frame(frame_id) {
            Ok(result) => vec![ok(request_id, command, ResponseBody::FrameRestart(result))],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}
