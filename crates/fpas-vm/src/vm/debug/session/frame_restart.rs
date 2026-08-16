//! Stopped-state orchestration for selected frame restart.

use super::*;
use crate::vm::debug::frame_restart::{DebugFrameRestartResult, apply, prepare, unsupported};
use crate::vm::debug::types::DebugTaskState;

impl DebugSession {
    /// Restart one selected live frame with its current arguments and captures.
    ///
    /// Locals and temporaries are cleared, younger frames are discarded, and
    /// execution remains stopped at the selected function entry.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns a stable state, frame, task, metadata, or register-state error.
    /// Failure leaves workers and inspection handles unchanged.
    pub fn restart_frame(
        &mut self,
        frame_id: u64,
    ) -> Result<DebugFrameRestartResult, DebugSessionError> {
        self.require_stopped("frame.restart")?;
        let task_id = self.task_for_frame(Some(frame_id))?;
        let frame = self
            .inspection_for_item(frame_id)?
            .stack(0, self.inspection_limits.max_frames)?
            .items
            .into_iter()
            .find(|frame| frame.id == frame_id)
            .ok_or_else(|| DebugSessionError {
                kind: DebugErrorKind::UnknownFrame,
                message: format!("debug frame {frame_id} is unknown or expired"),
                hint: "Request stack frames again for the current stop.".to_string(),
            })?;
        if task_id != self.last_stop.task_id {
            return Err(unsupported(
                format!(
                    "frame restart is not available for task {task_id}; the current stop belongs to task {}",
                    self.last_stop.task_id
                ),
                "Select a current frame from the task that caused the all-stop event.",
            ));
        }
        if !matches!(
            self.runtime.task_state(task_id),
            Some(DebugTaskState::Runnable | DebugTaskState::Running)
        ) {
            return Err(unsupported(
                "frame restart requires a runnable live task",
                "Wait until the selected task causes a normal stopped-state event.",
            ));
        }
        let prepared = {
            let worker = self
                .runtime
                .worker(task_id)
                .ok_or_else(|| unknown_task(task_id))?;
            prepare(worker, frame.depth)?
        };
        let discarded_frames = prepared.discarded_frames();
        let worker = self
            .runtime
            .worker_mut(task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        apply(worker, prepared);
        self.last_stop = stop_at_worker(
            &self.executable,
            worker,
            DebugStopReason::Pause,
            Vec::new(),
            None,
        );
        self.invalidate_inspection();
        self.refresh_inspection();
        self.inspection_task_id = task_id;
        let restarted = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?
            .stack(0, 1)?
            .items
            .into_iter()
            .next()
            .unwrap_or_else(|| unreachable!("restarted live task exposes its active frame"));
        Ok(DebugFrameRestartResult {
            task_id,
            frame: restarted,
            discarded_frames,
        })
    }
}
