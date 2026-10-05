//! Type-check projects and workspaces without running the VM.
//!
//! Documentation: `docs/pascal/program-structure/cli.md`,
//! `docs/pascal/tools/diagnostics.md`.

use std::fs;
use std::path::Path;

use crate::cli_input::{CliConfig, CliInput};
use crate::cli_output::{CliFailure, Reporter};
use crate::cli_paths::collect_fpas_files_in_dir;
use fpas_diagnostics::codes::{CLI_INPUT_UNSUPPORTED, PROJECT_DIRECTORY_READ_FAILED};
use fpas_diagnostics::{DiagnosticSeverity, FileDiagnostic};
use fpas_project as project;

mod directory;

/// Checks sources from CLI-resolved input without execution.
pub(crate) fn check_cli(
    config: CliConfig,
    standard_library: Option<&project::StandardLibrary>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    match config.input {
        CliInput::SourceFile(path) if path.is_dir() => {
            check_source_directory(&path, standard_library, reporter)
        }
        CliInput::SourceFile(path) => check_source_file(&path, standard_library, reporter),
        CliInput::ProjectFile(path) => check_project_file(&path, standard_library, reporter),
        CliInput::WorkspaceFile(path) => check_workspace_file(&path, standard_library, reporter),
        CliInput::CompiledProgramFile(path) => reporter.failure(
            &CliFailure::new(
                CLI_INPUT_UNSUPPORTED,
                format!("Cannot check compiled program `{}`.", path.display()),
            )
            .with_help("Check its source project instead."),
        ),
    }
}

fn check_source_directory(
    dir: &Path,
    standard_library: Option<&project::StandardLibrary>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let files = match collect_fpas_files_in_dir(dir) {
        Ok(files) => files,
        Err(message) => {
            return reporter.failure(&CliFailure::from_message(
                PROJECT_DIRECTORY_READ_FAILED,
                &message,
            ));
        }
    };
    if files.is_empty() {
        return reporter.failure(
            &CliFailure::new(
                CLI_INPUT_UNSUPPORTED,
                format!("No `.fpas` files found under `{}`.", dir.display()),
            )
            .with_help("Pass a source file, project, or workspace path."),
        );
    }

    directory::check_source_set(&files, standard_library, reporter)
}

fn check_source_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    if let Some(standard_library) = standard_library {
        return match crate::project_build::build_test_program(
            path,
            &[],
            &project::ProjectLinkMeta::default(),
            Some(standard_library),
        ) {
            Ok(_) => 0,
            Err(failure) => reporter.failure(&failure),
        };
    }
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return reporter.failure(&crate::project_build::source_read_failure(path, &error));
        }
    };

    check_parsed_source(path, &source, reporter)
}

fn check_project_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let loaded = match project::load_project(path) {
        Ok(loaded) => loaded,
        Err(error) => return reporter.failure(&error.into()),
    };
    reporter.records(&loaded.warnings);

    let checked = match loaded.kind {
        project::ProjectKind::Program => {
            crate::project_build::build_program(&loaded, standard_library).map(|_| ())
        }
        project::ProjectKind::Library => {
            crate::project_build::check_library(&loaded, standard_library)
        }
        project::ProjectKind::Test => {
            crate::project_build::check_test_project(&loaded, standard_library)
        }
    };
    match checked {
        Ok(()) => 0,
        Err(failure) => reporter.failure(&failure),
    }
}

fn check_workspace_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let workspace = match project::load_workspace(path) {
        Ok(workspace) => workspace,
        Err(error) => return reporter.failure(&error.into()),
    };

    let mut exit_code = 0;
    for member in &workspace.member_projects {
        let member_exit = check_project_file(member, standard_library, reporter);
        if member_exit != 0 {
            exit_code = member_exit;
        }
    }

    exit_code
}

fn check_parsed_source(path: &Path, source: &str, reporter: &mut Reporter<'_>) -> i32 {
    let (program, parse_errors) = fpas_parser::parse(source);
    let has_errors = parse_errors
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().severity == DiagnosticSeverity::Error);

    for diagnostic in &parse_errors {
        reporter.record(&FileDiagnostic::new(
            diagnostic.as_diagnostic().clone(),
            Some(path.to_path_buf()),
        ));
    }

    if has_errors {
        return 1;
    }

    match fpas_compiler::compile(&program) {
        Ok(_chunk) => 0,
        Err(diagnostics) => reporter.failure(&CliFailure::from_diagnostics(path, &diagnostics)),
    }
}
