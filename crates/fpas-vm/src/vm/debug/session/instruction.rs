//! Stopped-state orchestration for rejected instruction-pointer changes.

use super::*;
use crate::vm::debug::instruction::unsupported;

impl DebugSession {
    /// Reject one requested instruction-pointer change without mutating the debuggee.
    ///
    /// A supplied frame must belong to the current stop so expired handles fail as
    /// unknown frames. Every concrete destination, including the current
    /// instruction and other same-function sequence points, is rejected.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub fn set_instruction(
        &mut self,
        frame_id: Option<u64>,
        _instruction: Option<u32>,
    ) -> DebugSessionError {
        if let Err(error) = self.require_stopped("instruction.set") {
            return error;
        }
        if let Some(frame_id) = frame_id {
            if let Err(error) = self.task_for_frame(Some(frame_id)) {
                return error;
            }
            match self.inspection_for_item(frame_id).and_then(|inspection| {
                inspection
                    .stack(0, self.inspection_limits.max_frames)
                    .map(|stack| {
                        stack
                            .items
                            .into_iter()
                            .find(|frame| frame.id == frame_id)
                            .is_some()
                    })
            }) {
                Ok(true) => {}
                Ok(false) | Err(_) => {
                    return DebugSessionError {
                        kind: DebugErrorKind::UnknownFrame,
                        message: format!("debug frame {frame_id} is unknown or expired"),
                        hint: "Request stack frames again for the current stop.".to_string(),
                    };
                }
            }
        }
        unsupported()
    }
}
