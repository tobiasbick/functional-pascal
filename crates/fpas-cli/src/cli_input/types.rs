//! CLI configuration types resolved from argv.

use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum CliInput {
    SourceFile(PathBuf),
    ProjectFile(PathBuf),
    WorkspaceFile(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CliConfig {
    pub input: CliInput,
    pub program_args: Vec<String>,
    pub standard_library: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TestReportFormat {
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FmtCliConfig {
    pub explicit_args: Vec<String>,
    pub cwd: PathBuf,
    pub check_only: bool,
    pub stdout: bool,
    pub list_changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TestCliConfig {
    pub input: CliInput,
    pub cwd: PathBuf,
    pub fail_fast: bool,
    pub list_only: bool,
    pub script_path: Option<PathBuf>,
    pub filter: Option<String>,
    pub report: Option<TestReportFormat>,
    pub timeout: Option<Duration>,
    pub jobs: usize,
    pub strict: bool,
    pub standard_library: Option<PathBuf>,
}

/// Result of parsing CLI arguments before loading sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedCli {
    Run(CliConfig),
    Check(CliConfig),
    Fmt(FmtCliConfig),
    Test(TestCliConfig),
    Help,
    Version,
}
