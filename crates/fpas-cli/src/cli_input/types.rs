//! CLI configuration types resolved from argv.

use std::path::PathBuf;
use std::time::Duration;

/// Scaffold kind selected by `fpas init`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InitKind {
    Project,
    Library,
    Workspace,
}

impl InitKind {
    /// Returns the stable CLI spelling for this scaffold kind.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Library => "library",
            Self::Workspace => "workspace",
        }
    }
}

/// Machine-readable output format selected for `fpas init`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InitReportFormat {
    Json,
}

/// Fully resolved configuration for an `fpas init` invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InitCliConfig {
    pub cwd: PathBuf,
    pub kind: InitKind,
    pub name: String,
    pub root: PathBuf,
    pub library_unit: Option<String>,
    pub dry_run: bool,
    pub report: Option<InitReportFormat>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum CliInput {
    SourceFile(PathBuf),
    ProjectFile(PathBuf),
    WorkspaceFile(PathBuf),
    CompiledProgramFile(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CliConfig {
    pub input: CliInput,
    pub program_args: Vec<String>,
    pub standard_library: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DebugProtocol {
    Jsonl,
    Dap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DebugCliConfig {
    pub cwd: PathBuf,
    pub input: CliInput,
    pub program_args: Vec<String>,
    pub standard_library: Option<PathBuf>,
    pub protocol: DebugProtocol,
    pub commands: Option<PathBuf>,
    pub source_root: Option<PathBuf>,
    pub timeout: Duration,
    pub instruction_limit: u64,
    pub output_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuildCliConfig {
    pub input: CliInput,
    pub standard_library: Option<PathBuf>,
    pub executable: bool,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TestReportFormat {
    Json,
}

/// Help page selected by the top-level command or one of its subcommands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HelpTopic {
    General,
    Init,
    InitProject,
    InitLibrary,
    InitWorkspace,
    Build,
    Run,
    Debug,
    Check,
    Fmt,
    Test,
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
    Init(InitCliConfig),
    Build(BuildCliConfig),
    Run(CliConfig),
    Debug(DebugCliConfig),
    Check(CliConfig),
    Fmt(FmtCliConfig),
    Test(TestCliConfig),
    Help(HelpTopic),
    Version,
}
