//! Format `.fpas` compilation units in place.
//!
//! Documentation: `docs/pascal/fmt-style.md`, `docs/pascal/program-structure/projects.md`

mod paths;

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::cli_input::FmtCliConfig;
use crate::cli_run::render_cli_diagnostic;
use fpas_diagnostics::DiagnosticSeverity;
use fpas_fmt::format_source;
use fpas_parser::parse_compilation_unit;

/// Exit code when `--check` finds files that would change.
pub(crate) const EXIT_WOULD_CHANGE: i32 = 2;

/// Formats sources from CLI-resolved input.
pub(crate) fn format_cli(
    config: FmtCliConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let paths = match paths::collect_format_paths(&config.cwd, &config.explicit_args, stderr) {
        Ok(paths) => paths,
        Err(exit_code) => return exit_code,
    };

    if config.stdout {
        if paths.len() != 1 {
            let _ = writeln!(
                stderr,
                "`fpas fmt --stdout` requires exactly one `.fpas` file.\n  help: Pass a single source path."
            );
            return 1;
        }
    }

    let mut exit_code = 0;
    let mut would_change = false;

    for path in &paths {
        match format_source_file(path, &config, stdout, stderr) {
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

fn format_source_file(
    path: &Path,
    config: &FmtCliConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<bool, i32> {
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

    let formatted = format_source(&source, &unit);
    let changed = normalize_newlines(&source) != formatted;

    if config.list_changed && config.check_only && changed {
        let _ = writeln!(stdout, "{}", path.display());
    }

    if config.stdout {
        let _ = write!(stdout, "{formatted}");
        return Ok(changed);
    }

    if changed && !config.check_only {
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
