//! Host-produced verified image and source bundle for one reload attempt.

use crate::DebugSourceContent;

pub(crate) type DebugReloadProvider =
    Box<dyn FnMut() -> Result<ReloadedDebugTarget, fpas_vm::DebugSessionError> + Send + 'static>;

/// One rebuilt executable and its exact verified portable source contents.
pub struct ReloadedDebugTarget {
    executable: fpas_bytecode::VerifiedExecutable,
    sources: Vec<DebugSourceContent>,
}

impl ReloadedDebugTarget {
    /// Construct a reload candidate without retained source text.
    #[must_use]
    pub fn new(executable: fpas_bytecode::VerifiedExecutable) -> Self {
        Self {
            executable,
            sources: Vec::new(),
        }
    }

    /// Attach source text verified by the same host build as the executable.
    #[must_use]
    pub fn with_sources(mut self, sources: Vec<DebugSourceContent>) -> Self {
        self.sources = sources;
        self
    }

    pub(crate) fn into_parts(self) -> (fpas_bytecode::VerifiedExecutable, Vec<DebugSourceContent>) {
        (self.executable, self.sources)
    }
}
