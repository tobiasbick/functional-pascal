//! Verified debugger launch input prepared by the owning CLI or editor adapter.

/// Verified source content available to editor protocol clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSourceContent {
    /// Portable path stored in debugger metadata.
    pub path: String,
    /// Exact UTF-8 source text whose identity was verified before launch.
    pub content: String,
}

/// One verified executable, its process arguments, and optional source content.
pub struct PreparedDebugTarget {
    executable: fpas_bytecode::VerifiedExecutable,
    arguments: Vec<String>,
    execution_limits: fpas_vm::DebugExecutionLimits,
    sources: Vec<DebugSourceContent>,
}

impl PreparedDebugTarget {
    /// Construct a protocol-ready debugger target.
    #[must_use]
    pub fn new(executable: fpas_bytecode::VerifiedExecutable, arguments: Vec<String>) -> Self {
        Self {
            executable,
            arguments,
            execution_limits: fpas_vm::DebugExecutionLimits::default(),
            sources: Vec::new(),
        }
    }

    /// Override controlled-execution limits for this launch.
    #[must_use]
    pub const fn with_execution_limits(
        mut self,
        execution_limits: fpas_vm::DebugExecutionLimits,
    ) -> Self {
        self.execution_limits = execution_limits;
        self
    }

    /// Return portable debugger source paths embedded in the executable.
    #[must_use]
    pub fn source_paths(&self) -> Vec<String> {
        let executable = self.executable.executable();
        executable
            .source_map
            .sources
            .iter()
            .filter_map(|source| executable.strings.get(*source).map(str::to_owned))
            .collect()
    }

    /// Attach exact verified source text for DAP `source` requests.
    #[must_use]
    pub fn with_sources(mut self, sources: Vec<DebugSourceContent>) -> Self {
        self.sources = sources;
        self
    }

    pub(crate) fn sources(&self) -> &[DebugSourceContent] {
        &self.sources
    }

    pub(crate) const fn execution_limits(&self) -> fpas_vm::DebugExecutionLimits {
        self.execution_limits
    }

    pub(crate) fn into_session(self) -> Result<fpas_vm::DebugSession, fpas_vm::DebugSessionError> {
        fpas_vm::DebugSession::with_limits(
            self.executable,
            self.arguments,
            fpas_vm::DebugInspectionLimits::default(),
            self.execution_limits,
        )
    }
}
