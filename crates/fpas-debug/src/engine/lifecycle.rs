//! Rejected debugger task create and restart.

use super::record::DebugRecord;
use super::reply::{invalid_state, session_error};
use super::{DebugEngine, DebugStatus};

impl DebugEngine {
    pub(super) fn create_task(&mut self, request_id: u64, command: &str) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        vec![session_error(request_id, command, session.create_task())]
    }

    pub(super) fn restart_task(
        &mut self,
        request_id: u64,
        command: &str,
        task_id: Option<u64>,
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
            session.restart_task(task_id),
        )]
    }
}
