//! Shared source-set validation for `fpas check <directory>`.

use std::fs;
use std::path::PathBuf;

use fpas_diagnostics::FileDiagnostic;
use fpas_parser::{CompilationUnit, parse_compilation_unit};
use fpas_project::{ProjectLinkMeta, StandardLibrary};

use crate::cli_output::Reporter;

/// Checks all programs and units discovered in one directory tree as one source set.
pub(super) fn check_source_set(
    files: &[PathBuf],
    standard_library: Option<&StandardLibrary>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let Some((programs, units)) = classify_sources(files, reporter) else {
        return 1;
    };
    let link_meta = ProjectLinkMeta::default();

    if !units.is_empty()
        && let Err(failure) =
            crate::project_build::check_source_units(&units, &link_meta, standard_library)
    {
        return reporter.failure(&failure);
    }

    let mut exit_code = 0;
    for program in programs {
        if let Err(failure) = crate::project_build::check_source_program(
            &program,
            &units,
            &link_meta,
            standard_library,
        ) {
            exit_code = reporter.failure(&failure);
        }
    }
    exit_code
}

fn classify_sources(
    files: &[PathBuf],
    reporter: &mut Reporter<'_>,
) -> Option<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut programs = Vec::new();
    let mut units = Vec::new();
    let mut has_errors = false;

    for path in files {
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                reporter.failure(&crate::project_build::source_read_failure(path, &error));
                has_errors = true;
                continue;
            }
        };
        let (parsed, diagnostics) = parse_compilation_unit(&source);
        let mut file_has_errors = false;
        for diagnostic in &diagnostics {
            let diagnostic = diagnostic.as_diagnostic();
            file_has_errors |= diagnostic.is_error();
            reporter.record(&FileDiagnostic::new(diagnostic.clone(), Some(path.clone())));
        }
        if file_has_errors {
            has_errors = true;
            continue;
        }
        match parsed {
            CompilationUnit::Program(_) => programs.push(path.clone()),
            CompilationUnit::Unit(_) => units.push(path.clone()),
        }
    }

    (!has_errors).then_some((programs, units))
}
