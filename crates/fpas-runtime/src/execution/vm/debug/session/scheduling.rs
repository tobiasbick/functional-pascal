//! Construction-time scheduling choice and read-only coverage metadata.

use super::*;

impl DebugSession {
    /// Construct controlled batch execution with an optional seed and virtual task clock.
    ///
    /// # Errors
    /// Returns an initialization error before any program instruction executes.
    pub fn with_limits_and_schedule(
        executable: VerifiedExecutable,
        arguments: Vec<String>,
        inspection_limits: DebugInspectionLimits,
        execution_limits: DebugExecutionLimits,
        seed: Option<u64>,
    ) -> Result<Self, DebugSessionError> {
        Self::with_task_clock(
            executable,
            arguments,
            inspection_limits,
            execution_limits,
            Arc::new(if seed.is_some() {
                TaskClock::manual()
            } else {
                TaskClock::realtime()
            }),
            seed,
        )
    }

    /// Read scheduling metadata without dispatch, readiness polling, or time advancement.
    #[must_use]
    pub fn schedule_report(&self) -> crate::execution::vm::debug::scheduling::DebugScheduleReport {
        self.runtime.schedule_report()
    }
}
