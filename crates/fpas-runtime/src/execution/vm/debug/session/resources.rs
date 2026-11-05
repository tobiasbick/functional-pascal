//! All debugger execution and mutation uses the live session's resource account.

use super::*;

impl DebugSession {
    pub(super) fn resource_budget(&self) -> Arc<crate::resources::Budget> {
        self.runtime
            .worker(0)
            .map(|worker| worker.hosted.budget.clone())
            .unwrap_or_default()
    }

    pub(super) fn budget_failure<T>(
        error: crate::resources::Exhausted,
    ) -> Result<T, DebugSessionError> {
        use crate::resources::Exhausted;
        Err(DebugSessionError {
            kind: match error {
                Exhausted::Heap { .. } => DebugErrorKind::HeapLimit,
                Exhausted::Instructions { .. } => DebugErrorKind::InstructionLimit,
                Exhausted::Timeout => DebugErrorKind::ExecutionTimeout,
                Exhausted::Output { .. } => DebugErrorKind::OutputLimit,
            },
            message: error.to_string(),
            hint: "Increase the execution budget or reduce the program's resource use.".into(),
        })
    }

    pub(super) fn with_resource_budget<T>(
        &self,
        run: impl FnOnce() -> Result<T, DebugSessionError>,
    ) -> Result<T, DebugSessionError> {
        let resources = self.resource_budget();
        let _resources = crate::resources::Scope::enter(&resources);
        crate::resources::catch(run).unwrap_or_else(Self::budget_failure)
    }

    pub(super) fn enforce_resource_failure(&mut self) -> Result<(), DebugSessionError> {
        if let Some(error) = self.resource_budget().failure() {
            self.state = DebugSessionState::Failed;
            self.refresh_inspection();
            return Self::budget_failure(error);
        }
        Ok(())
    }

    /// Observe execution-wide resource accounting for the debugger session.
    #[must_use]
    pub fn execution_usage(&self) -> crate::ExecutionUsage {
        self.resource_budget().usage()
    }
}
