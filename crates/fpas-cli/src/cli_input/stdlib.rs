//! Explicit distribution-staging arguments, without source discovery.
use super::{HelpTopic, ResolvedCli};
use crate::cli_paths::normalize_input_path;
use std::path::{Path, PathBuf};

/// Disposable source staging tree and final distribution destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StdlibCliConfig {
    pub source: PathBuf,
    pub output: PathBuf,
    pub dry_run: bool,
}

const USAGE: &str = "Usage: fpas stdlib stage --source <dir> --output <dir> [--dry-run]\n  help: `fpas stdlib stage --help` shows the staging contract.";

pub(super) fn resolve(args: &[String], cwd: &Path) -> Result<ResolvedCli, String> {
    if matches!(args, [help] if help == "--help" || help == "-h") {
        return Ok(ResolvedCli::Help(HelpTopic::Stdlib));
    }
    let Some((command, args)) = args.split_first() else {
        return Err(USAGE.into());
    };
    if command != "stage" {
        return Err(format!(
            "Unknown standard-library command `{command}`.\n{USAGE}"
        ));
    }
    if matches!(args, [help] if help == "--help" || help == "-h") {
        return Ok(ResolvedCli::Help(HelpTopic::Stdlib));
    }
    let mut source = None;
    let mut output = None;
    let mut dry_run = false;
    let mut args = args.iter();
    while let Some(argument) = args.next() {
        let destination = match argument.as_str() {
            "--source" => &mut source,
            "--output" => &mut output,
            "--dry-run" if !dry_run => {
                dry_run = true;
                continue;
            }
            _ => return Err(format!("Unexpected argument `{argument}`.\n{USAGE}")),
        };
        if destination.is_some() {
            return Err(format!("Duplicate option `{argument}`.\n{USAGE}"));
        }
        let value = args
            .next()
            .filter(|value| !value.starts_with("--") && !value.is_empty())
            .ok_or_else(|| format!("Missing directory after `{argument}`.\n{USAGE}"))?;
        *destination = Some(normalize_input_path(value, cwd));
    }
    Ok(ResolvedCli::StageStdlib(StdlibCliConfig {
        source: source.ok_or_else(|| format!("Missing --source.\n{USAGE}"))?,
        output: output.ok_or_else(|| format!("Missing --output.\n{USAGE}"))?,
        dry_run,
    }))
}

pub(super) const HELP: &str = "\
Prepare the standard library using this compiler's artifact identity.

Usage:
  fpas stdlib stage --source <dir> --output <dir> [--dry-run]

Options:
  --source <dir>  Disposable staging tree containing stdlib.fpasprj
  --output <dir>  Distribution library directory to replace
  --dry-run       Print resolved paths without reading or writing either tree
  -h, --help      Print this help

Staging deletes compiled artifacts in the source tree and recompiles its units.
The output is replaced only after compilation succeeds. Paths must be separate,
non-nested directories without symbolic links or reparse points.
Use the Cargo-staged library, never the repository's source library.

Example:
  fpas stdlib stage --source target/release/lib --output bin/lib
";
