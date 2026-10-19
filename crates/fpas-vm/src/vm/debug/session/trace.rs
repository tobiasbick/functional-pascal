//! Pre-execution trace setup and detached terminal history.

use super::super::trace::{DebugTraceReport, TraceHistory};
use super::*;

impl DebugSession {
    /// Atomically bind a bounded trace request before any bytecode has run.
    ///
    /// # Errors
    /// Rejects invalid, unsupported, oversized or late requests without installing watches.
    pub fn install_trace(
        &mut self,
        selectors: &[String],
        limit: usize,
    ) -> Result<(), DebugSessionError> {
        if self.state != DebugSessionState::Stopped || self.runtime.instruction_count() != 0 {
            return Err(DebugSessionError::invalid_state(
                "trace.install",
                self.state,
            ));
        }
        let history = TraceHistory::new(self.executable.executable(), selectors, limit)
            .map_err(|message| DebugSessionError {
                kind: DebugErrorKind::VariablePathUnsupported,
                message,
                hint: "Trace a scalar global path or exact Routine::Name local/parameter/capture path, using stored fields, literal indexes and full executable metadata names.".into(),
            })?;
        if selectors.is_empty() {
            self.trace = None;
        } else {
            self.trace = Some(Arc::new(Mutex::new(history)));
        }
        self.runtime.set_trace(self.trace.clone());
        Ok(())
    }

    /// Detach the retained history after execution; no live values or handles escape.
    pub fn take_trace(&mut self) -> Option<DebugTraceReport> {
        let history = self.trace.take()?;
        self.runtime.set_trace(None);
        Some({
            let mut history = history.lock().unwrap_or_else(|p| p.into_inner());
            std::mem::take(&mut history.report)
        })
    }

    pub(in crate::vm::debug) fn trace_error(&self) -> Option<DebugSessionError> {
        let history = self
            .trace
            .as_ref()?
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let kind = match history.limit? {
            "trace_event_limit" => DebugErrorKind::TraceEventLimit,
            "trace_work_limit" => DebugErrorKind::TraceWorkLimit,
            "trace_byte_limit" => DebugErrorKind::TraceByteLimit,
            _ => DebugErrorKind::TraceValueLimit,
        };
        Some(DebugSessionError {
            kind,
            message: format!(
                "Trace collection ended at its {} bound.",
                history.limit.unwrap_or("trace_value_limit")
            ),
            hint:
                "Reduce trace selectors or program work, or raise --trace-limit within its ceiling."
                    .into(),
        })
    }
}
