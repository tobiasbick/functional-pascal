//! Shared source-set validation for `fpas check <directory>`.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use fpas_diagnostics::DiagnosticSeverity;
use fpas_parser::{CompilationUnit, parse_compilation_unit};
use fpas_project::{ProjectLinkMeta, StandardLibrary};

/// Checks all programs and units discovered in one directory tree as one source set.
pub(super) fn check_source_set(
    files: &[PathBuf],
    standard_library: Option<&StandardLibrary>,
    stderr: &mut dyn Write,
) -> i32 {
    let Some((programs, units)) = classify_sources(files, stderr) else {
        return 1;
    };
    let link_meta = ProjectLinkMeta::default();

    if !units.is_empty()
        && let Err(message) =
            crate::project_build::check_source_units(&units, &link_meta, standard_library)
    {
        let _ = writeln!(stderr, "{message}");
        return 1;
    }

    let mut exit_code = 0;
    for program in programs {
        if let Err(message) = crate::project_build::check_source_program(
            &program,
            &units,
            &link_meta,
            standard_library,
        ) {
            let _ = writeln!(stderr, "{message}");
            exit_code = 1;
        }
    }
    exit_code
}

fn classify_sources(
    files: &[PathBuf],
    stderr: &mut dyn Write,
) -> Option<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut programs = Vec::new();
    let mut units = Vec::new();
    let mut has_errors = false;

    for path in files {
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                let _ = writeln!(stderr, "Error reading `{}`: {error}", path.display());
                has_errors = true;
                continue;
            }
        };
        let (parsed, diagnostics) = parse_compilation_unit(&source);
        let path_text = path.to_string_lossy();
        for diagnostic in &diagnostics {
            let diagnostic = diagnostic.as_diagnostic();
            has_errors |= diagnostic.severity == DiagnosticSeverity::Error;
            super::emit_check_diagnostic(path_text.as_ref(), None, diagnostic, stderr);
        }
        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.as_diagnostic().severity == DiagnosticSeverity::Error)
        {
            continue;
        }
        match parsed {
            CompilationUnit::Program(_) => programs.push(path.clone()),
            CompilationUnit::Unit(_) => units.push(path.clone()),
        }
    }

    (!has_errors).then_some((programs, units))
}
