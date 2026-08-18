//! Verified debugger launch and reload input prepared by the owning host.

pub(crate) use crate::target_reload::DebugReloadProvider;
pub use crate::target_reload::ReloadedDebugTarget;

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
    reloader: Option<DebugReloadProvider>,
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
            reloader: None,
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

    /// Attach the host-owned operation that rebuilds this exact launch target.
    ///
    /// The debugger invokes the provider only for explicit reload requests and
    /// accepts only verified executables returned by it.
    #[must_use]
    pub fn with_reloader<F>(mut self, reloader: F) -> Self
    where
        F: FnMut() -> Result<ReloadedDebugTarget, fpas_vm::DebugSessionError> + Send + 'static,
    {
        self.reloader = Some(Box::new(reloader));
        self
    }

    pub(crate) fn sources(&self) -> &[DebugSourceContent] {
        &self.sources
    }

    pub(crate) const fn execution_limits(&self) -> fpas_vm::DebugExecutionLimits {
        self.execution_limits
    }

    pub(crate) fn take_reloader(&mut self) -> Option<DebugReloadProvider> {
        self.reloader.take()
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
