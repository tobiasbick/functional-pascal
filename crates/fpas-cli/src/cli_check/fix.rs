//! `fpas check --fix`: applies machine-applicable fixes to the checked sources, pass after
//! pass, before the final check reports what remains.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md` (Fixes),
//! `docs/pascal/program-structure/cli.md`.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use fpas_diagnostics::codes::CLI_OUTPUT_FAILED;
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
/// Only `.fpas` files under the checked input are edited, never the standard library.
/// Returns a write failure, or the summary.
pub(super) fn apply_fixes(
    input: &CliInput,
    standard_library: Option<&project::StandardLibrary>,
    check: impl Fn(&CliInput, &mut Reporter<'_>) -> i32,
) -> Result<FixSummary, CliFailure> {
    let scope = EditScope::new(input, standard_library);
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

/// The sources `--fix` may edit: the checked file, or the files under the checked directory,
/// project or workspace, excluding the standard library.
struct EditScope {
    root: Option<PathBuf>,
    single_file: bool,
    excluded: Vec<PathBuf>,
}

impl EditScope {
    fn new(input: &CliInput, standard_library: Option<&project::StandardLibrary>) -> Self {
        let (root, single_file) = match input {
            CliInput::SourceFile(path) if path.is_dir() => (Some(path.clone()), false),
            CliInput::SourceFile(path) => (Some(path.clone()), true),
            CliInput::ProjectFile(path) | CliInput::WorkspaceFile(path) => {
                let directory = path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty());
                (
                    Some(directory.unwrap_or(Path::new(".")).to_path_buf()),
                    false,
                )
            }
            CliInput::CompiledProgramFile(_) => (None, false),
        };
        Self {
            root: root.and_then(|root| canonical(&root)),
            single_file,
            excluded: standard_library
                .map(|library| {
                    library
                        .source_files()
                        .iter()
                        .filter_map(|path| canonical(path))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    fn allows(&self, path: &Path) -> bool {
        let Some(root) = &self.root else {
            return false;
        };
        let Some(path) = canonical(path) else {
            return false;
        };
        let inside = if self.single_file {
            &path == root
        } else {
            path.starts_with(root)
        };
        inside
            && path
                .extension()
                .is_some_and(|extension| extension == "fpas")
            && !self.excluded.contains(&path)
    }
}

fn canonical(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path).ok()
}
