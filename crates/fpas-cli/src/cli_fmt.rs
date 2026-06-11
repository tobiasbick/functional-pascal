//! Format `.fpas` compilation units in place.
//!
//! Documentation: `docs/future/formater/style.md`, `docs/pascal/10-projects.md`

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cli_input::{CliInput, FmtCliConfig};
use crate::cli_run::render_cli_diagnostic;
use fpas_diagnostics::DiagnosticSeverity;
use fpas_fmt::format_compilation_unit;
use fpas_parser::parse_compilation_unit;
use fpas_project as project;

/// Exit code when `--check` finds files that would change.
pub(crate) const EXIT_WOULD_CHANGE: i32 = 2;

/// Formats sources from CLI-resolved input.
pub(crate) fn format_cli(config: FmtCliConfig, stderr: &mut dyn Write) -> i32 {
    let paths = match collect_format_paths(&config.input, stderr) {
        Ok(paths) => paths,
        Err(exit_code) => return exit_code,
    };

    let mut exit_code = 0;
    let mut would_change = false;

    for path in paths {
        match format_source_file(&path, config.check_only, stderr) {
            Ok(changed) => {
                if changed {
                    would_change = true;
                }
            }
            Err(code) => exit_code = code,
        }
    }

    if exit_code != 0 {
        return exit_code;
    }
    if config.check_only && would_change {
        return EXIT_WOULD_CHANGE;
    }

    0
}

fn collect_format_paths(input: &CliInput, stderr: &mut dyn Write) -> Result<Vec<PathBuf>, i32> {
    match input {
        CliInput::SourceFile(path) => Ok(vec![path.clone()]),
        CliInput::ProjectFile(path) => collect_project_paths(path, stderr),
        CliInput::WorkspaceFile(path) => collect_workspace_paths(path, stderr),
    }
}

fn collect_project_paths(path: &Path, stderr: &mut dyn Write) -> Result<Vec<PathBuf>, i32> {
    let loaded = match project::load_project(path) {
        Ok(loaded) => loaded,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return Err(1);
        }
    };

    for warning in &loaded.warnings {
        let _ = writeln!(stderr, "warning: {warning}");
    }

    Ok(loaded.source_files)
}

fn collect_workspace_paths(path: &Path, stderr: &mut dyn Write) -> Result<Vec<PathBuf>, i32> {
    let workspace = match project::load_workspace(path) {
        Ok(workspace) => workspace,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return Err(1);
        }
    };

    let mut paths = Vec::new();
    for member in &workspace.member_projects {
        paths.extend(collect_project_paths(member, stderr)?);
    }

    Ok(paths)
}

fn format_source_file(path: &Path, check_only: bool, stderr: &mut dyn Write) -> Result<bool, i32> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            let _ = writeln!(stderr, "Error reading `{}`: {error}", path.display());
            return Err(1);
        }
    };

    let path_text = path.to_string_lossy();
    let (unit, parse_errors) = parse_compilation_unit(&source);
    let has_errors = parse_errors
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().severity == DiagnosticSeverity::Error);

    for diagnostic in &parse_errors {
        if !emit_fmt_diagnostic(path_text.as_ref(), diagnostic.as_diagnostic(), stderr) {
            return Err(1);
        }
    }

    if has_errors {
        return Err(1);
    }

    let formatted = format_compilation_unit(&unit);
    let changed = normalize_newlines(&source) != formatted;

    if changed && !check_only {
        if let Err(error) = fs::write(path, &formatted) {
            let _ = writeln!(stderr, "Error writing `{}`: {error}", path.display());
            return Err(1);
        }
    }

    Ok(changed)
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn emit_fmt_diagnostic(
    path: &str,
    diagnostic: &fpas_diagnostics::Diagnostic,
    stderr: &mut dyn Write,
) -> bool {
    writeln!(stderr, "{}", render_cli_diagnostic(path, diagnostic)).is_ok()
}
