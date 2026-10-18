//! `fpas check --fix`: applies machine-applicable fixes to the checked sources, pass after
//! pass, before the final check reports what remains.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md` (Fixes),
//! `docs/pascal/program-structure/cli.md`.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use fpas_diagnostics::codes::{CLI_OUTPUT_FAILED, PROJECT_DIRECTORY_READ_FAILED};
use fpas_diagnostics::{Fix, apply_machine_applicable};
use fpas_project as project;

use crate::cli_input::CliInput;
use crate::cli_output::{CliFailure, DiagnosticFormat, Reporter};

/// Recovery stops at some habits, and fixing one can reveal the next, so `--fix` checks again
/// after every pass; this bounds the passes.
const MAX_FIX_PASSES: usize = 10;

/// Fixes applied by `--fix`, reported once after the final check.
#[derive(Debug, Default)]
pub(super) struct FixSummary {
    fixes: usize,
    files: usize,
}

impl fmt::Display for FixSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fixes = if self.fixes == 1 { "fix" } else { "fixes" };
        let files = if self.files == 1 { "file" } else { "files" };
        write!(
            formatter,
            "applied {} {fixes} in {} {files}",
            self.fixes, self.files
        )
    }
}

/// Applies fixes until a pass applies none or the pass limit is reached.
///
/// Only sources owned by the checked input are edited, never dependency-only projects or the
/// standard library. Returns a source-selection or write failure, or the summary.
pub(super) fn apply_fixes(
    input: &CliInput,
    standard_library: Option<&project::StandardLibrary>,
    check: impl Fn(&CliInput, &mut Reporter<'_>) -> i32,
) -> Result<FixSummary, CliFailure> {
    let scope = EditScope::new(input, standard_library)?;
    let mut summary = FixSummary::default();
    let mut changed_files = Vec::new();
    for _ in 0..MAX_FIX_PASSES {
        let mut discarded = Vec::new();
        let mut reporter = Reporter::new(DiagnosticFormat::Text, &mut discarded).collecting();
        check(input, &mut reporter);
        let mut fixes: BTreeMap<PathBuf, Vec<Fix>> = BTreeMap::new();
        for record in reporter.take_collected() {
            if let Some(path) = record.path.filter(|path| scope.allows(path)) {
                fixes
                    .entry(path)
                    .or_default()
                    .extend(record.diagnostic.fixes);
            }
        }
        let mut applied_in_pass = 0;
        for (path, fixes) in fixes {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let (fixed, applied) = apply_machine_applicable(&text, &fixes);
            if applied == 0 {
                continue;
            }
            crate::source_publication::write_source(&path, &fixed).map_err(|error| {
                CliFailure::new(
                    CLI_OUTPUT_FAILED,
                    format!("Cannot write fixes to `{}`: {error}", path.display()),
                )
            })?;
            applied_in_pass += applied;
            if !changed_files.contains(&path) {
                changed_files.push(path);
            }
        }
        if applied_in_pass == 0 {
            break;
        }
        summary.fixes += applied_in_pass;
    }
    summary.files = changed_files.len();
    Ok(summary)
}

/// Exact sources owned by the selected input, excluding the standard library.
struct EditScope {
    files: HashSet<PathBuf>,
}

impl EditScope {
    fn new(
        input: &CliInput,
        standard_library: Option<&project::StandardLibrary>,
    ) -> Result<Self, CliFailure> {
        let paths = match input {
            CliInput::SourceFile(path) if path.is_dir() => {
                crate::cli_paths::collect_fpas_files_in_dir(path).map_err(|message| {
                    CliFailure::from_message(PROJECT_DIRECTORY_READ_FAILED, &message)
                })?
            }
            CliInput::SourceFile(path) => vec![path.clone()],
            CliInput::ProjectFile(path) => project::load_project_source_paths(path)?,
            CliInput::WorkspaceFile(path) => {
                let workspace = project::load_workspace(path)?;
                let mut paths = Vec::new();
                for member in &workspace.member_projects {
                    paths.extend(project::load_project_source_paths(member)?);
                }
                paths
            }
            CliInput::CompiledProgramFile(_) => Vec::new(),
        };
        let mut files = paths
            .iter()
            .filter_map(|path| canonical(path))
            .collect::<HashSet<_>>();
        if let Some(library) = standard_library {
            for path in library
                .source_files()
                .iter()
                .filter_map(|path| canonical(path))
            {
                files.remove(&path);
            }
        }
        Ok(Self { files })
    }

    fn allows(&self, path: &Path) -> bool {
        canonical(path).is_some_and(|path| self.files.contains(&path))
    }
}

fn canonical(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path).ok()
}
