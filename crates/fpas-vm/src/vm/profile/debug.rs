//! Shared profile ownership across debugger resume threads and session teardown.

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use fpas_bytecode::VerifiedExecutable;

use super::{ProfileReport, ProfileSink};
use crate::vm::{DebugRunResult, DebugSessionError, DebugStopReason};

/// Samples of one debug target, retained after its session and workers are dropped.
#[derive(Clone)]
pub struct DebugProfile {
    sink: Arc<ProfileSink>,
    executable: Arc<VerifiedExecutable>,
    outcome: Arc<AtomicU8>,
}

impl DebugProfile {
    /// Create a recorder for this exact executable before starting the session.
    #[must_use]
    pub fn new(executable: VerifiedExecutable) -> Self {
        Self {
            sink: Arc::default(),
            executable: Arc::new(executable),
            outcome: Arc::default(),
        }
    }

    /// Executable whose addresses and source identities the samples describe.
    #[must_use]
    pub fn executable(&self) -> &VerifiedExecutable {
        &self.executable
    }

    /// Resolve and drain samples after the debugger session has been dropped.
    /// Active workers retain local samples until their teardown.
    #[must_use]
    pub fn report(&self) -> ProfileReport {
        self.sink.report(self.executable.executable())
    }

    /// Last execution outcome, independent of protocol transport success.
    #[must_use]
    pub fn outcome(&self) -> &'static str {
        match self.outcome.load(Ordering::Relaxed) {
            1 => "stopped",
            2 => "completed",
            3 => "runtime_error",
            4 => "error",
            _ => "not_started",
        }
    }

    pub(in crate::vm) fn sink(&self) -> Arc<ProfileSink> {
        Arc::clone(&self.sink)
    }

    pub(in crate::vm) fn record_resume(&self, result: &Result<DebugRunResult, DebugSessionError>) {
        let outcome = match result {
            Ok(DebugRunResult::Terminated(_)) => 2,
            Ok(DebugRunResult::Stopped(stop)) if stop.reason == DebugStopReason::RuntimeError => 3,
            Ok(DebugRunResult::Stopped(_)) => 1,
            Err(_) => 4,
        };
        self.outcome.store(outcome, Ordering::Relaxed);
    }
}
