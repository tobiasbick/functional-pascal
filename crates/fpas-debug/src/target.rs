//! Verified debugger launch and reload input prepared by the owning host.

pub(crate) use crate::target_reload::DebugReloadProvider;
pub use crate::target_reload::ReloadedDebugTarget;

/// Verified source content available to editor protocol clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSourceContent {
    /// Portable path stored in debugger metadata.
    pub path: String,
    /// Optional original host path accepted from an editor for this source.
    pub original_path: Option<std::path::PathBuf>,
    /// Exact UTF-8 source text whose identity was verified before launch.
    pub content: String,
}

/// One verified executable, its process arguments, and optional source content.
pub struct PreparedDebugTarget {
    executable: fpas_runtime::VerifiedExecutable,
    arguments: Vec<String>,
    execution_limits: fpas_runtime::execution::DebugExecutionLimits,
    sources: Vec<DebugSourceContent>,
    reloader: Option<DebugReloadProvider>,
    schedule_seed: Option<u64>,
    collection_schedule: fpas_runtime::CollectionSchedule,
    profile: Option<fpas_runtime::execution::DebugProfile>,
}

impl PreparedDebugTarget {
    /// Construct a protocol-ready debugger target.
    #[must_use]
    pub fn new(executable: fpas_runtime::VerifiedExecutable, arguments: Vec<String>) -> Self {
        Self {
            executable,
            arguments,
            execution_limits: fpas_runtime::execution::DebugExecutionLimits::default(),
            sources: Vec::new(),
            reloader: None,
            schedule_seed: None,
            collection_schedule: fpas_runtime::CollectionSchedule::default(),
            profile: None,
        }
    }

    /// Configure collection thresholds for debugger tasks and helper domains.
    #[must_use]
    pub const fn with_collection_schedule(
        mut self,
        schedule: fpas_runtime::CollectionSchedule,
    ) -> Self {
        self.collection_schedule = schedule;
        self
    }

    /// Override controlled-execution limits for this launch.
    #[must_use]
    pub const fn with_execution_limits(
        mut self,
        execution_limits: fpas_runtime::execution::DebugExecutionLimits,
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

    /// Collect samples of this target, including workers on asynchronous resume threads.
    /// Read the returned recorder only after the owning debugger has been dropped.
    pub fn enable_profile(&mut self) -> fpas_runtime::execution::DebugProfile {
        self.profile
            .get_or_insert_with(|| {
                fpas_runtime::execution::DebugProfile::new(self.executable.clone())
            })
            .clone()
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
        F: FnMut() -> Result<ReloadedDebugTarget, fpas_runtime::execution::DebugSessionError>
            + Send
            + 'static,
    {
        self.reloader = Some(Box::new(reloader));
        self
    }

    pub(crate) fn sources(&self) -> &[DebugSourceContent] {
        &self.sources
    }

    pub(crate) const fn with_schedule_seed(mut self, seed: Option<u64>) -> Self {
        self.schedule_seed = seed;
        self
    }

    pub(crate) const fn execution_limits(&self) -> fpas_runtime::execution::DebugExecutionLimits {
        self.execution_limits
    }

    pub(crate) fn take_reloader(&mut self) -> Option<DebugReloadProvider> {
        self.reloader.take()
    }

    pub(crate) fn into_session(
        self,
    ) -> Result<fpas_runtime::execution::DebugSession, fpas_runtime::execution::DebugSessionError>
    {
        fpas_runtime::execution::DebugSession::with_limits_and_schedule(
            self.executable,
            self.arguments,
            fpas_runtime::execution::DebugInspectionLimits::default(),
            self.execution_limits,
            self.schedule_seed,
        )
        .map(|session| {
            let session = session.with_collection_schedule(self.collection_schedule);
            match self.profile {
                Some(profile) => session.with_profile(profile),
                None => session,
            }
        })
    }
}
