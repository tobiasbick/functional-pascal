//! Feasibility rejection for debugger task creation and task restart.

use super::*;

impl DebugSession {
    /// Reject debugger-created tasks without mutating workers or waiters.
    ///
    /// A successful spawn would execute bytecode and invent result handles that
    /// the stopped scheduler cannot prove. Use `go` in the program, then continue.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub fn create_task(&mut self) -> DebugSessionError {
        if let Err(error) = self.require_stopped("task.create") {
            return error;
        }
        DebugSessionError {
            kind: DebugErrorKind::TaskCreateUnsupported,
            message: "debugger task creation is not supported".to_string(),
            hint: "Spawn work with `go` in the program, then continue. The debugger cannot prove result handles, arguments, or waiter identity for a stopped-state spawn.".to_string(),
        }
    }

    /// Reject restarting a runtime task identity without mutating the debuggee.
    ///
    /// A supplied task must belong to the current catalog so stale IDs fail as
    /// unknown tasks. Frame restart remains the proven reconstruction path.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub fn restart_task(&mut self, task_id: Option<u64>) -> DebugSessionError {
        if let Err(error) = self.require_stopped("task.restart") {
            return error;
        }
        if let Some(task_id) = task_id
            && self.runtime.task_state(task_id).is_none()
        {
            return unknown_task(task_id);
        }
        DebugSessionError {
            kind: DebugErrorKind::TaskRestartUnsupported,
            message: "debugger task restart is not supported".to_string(),
            hint: "Restart a selected frame, or disconnect and launch again. Task restart would invent a new runtime identity.".to_string(),
        }
    }
}
