//! Type-check projects and workspaces without running the VM.
//!
//! Documentation: `docs/pascal/program-structure/cli.md`

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cli_input::{CliConfig, CliInput};
use crate::cli_paths::collect_fpas_files_in_dir;
use crate::cli_run::render_cli_diagnostic_with_sources;
use fpas_diagnostics::DiagnosticSeverity;
use fpas_project as project;

/// Checks sources from CLI-resolved input without execution.
pub(crate) fn check_cli(
    config: CliConfig,
    standard_library: Option<&project::StandardLibrary>,
    stderr: &mut dyn Write,
) -> i32 {
    match config.input {
        CliInput::SourceFile(path) if path.is_dir() => {
            check_source_directory(&path, standard_library, stderr)
        }
        CliInput::SourceFile(path) => check_source_file(&path, standard_library, stderr),
        CliInput::ProjectFile(path) => check_project_file(&path, standard_library, stderr),
        CliInput::WorkspaceFile(path) => check_workspace_file(&path, standard_library, stderr),
    }
}

fn check_source_directory(
    dir: &Path,
    standard_library: Option<&project::StandardLibrary>,
    stderr: &mut dyn Write,
) -> i32 {
    let files = collect_fpas_files_in_dir(dir);
    if files.is_empty() {
        let _ = writeln!(
            stderr,
            "No `.fpas` files found under `{}`.\n  help: Pass a source file, project, or workspace path.",
            dir.display()
        );
        return 1;
    }

    let mut exit_code = 0;
    for path in files {
        if check_source_file(&path, standard_library, stderr) != 0 {
            exit_code = 1;
        }
    }
    exit_code
}

fn check_source_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    stderr: &mut dyn Write,
) -> i32 {
    if let Some(standard_library) = standard_library {
        return match crate::project_build::build_test_program(
            path,
            &[],
            &project::ProjectLinkMeta::default(),
            Some(standard_library),
        ) {
            Ok(_) => 0,
            Err(message) => {
                let _ = writeln!(stderr, "{message}");
                1
            }
        };
    }
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            let _ = writeln!(stderr, "Error reading `{}`: {error}", path.display());
            return 1;
        }
    };

    let path_text = path.to_string_lossy();
    check_parsed_source(path_text.as_ref(), &source, None, stderr)
}

fn check_project_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    stderr: &mut dyn Write,
) -> i32 {
    let loaded = match project::load_project(path) {
        Ok(loaded) => loaded,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 1;
        }
    };

    for warning in &loaded.warnings {
        let _ = writeln!(stderr, "warning: {warning}");
    }

    match loaded.kind {
        project::ProjectKind::Program => {
            if loaded.main.is_none() {
                let _ = writeln!(
                    stderr,
                    "Project is missing `project.main`.\n  help: Set `main = \"src/main.fpas\"` in `[project]`."
                );
                return 1;
            }
            match crate::project_build::build_program(&loaded, standard_library) {
                Ok(program) => program,
                Err(message) => {
                    let _ = writeln!(stderr, "{message}");
                    return 1;
                }
            };
            0
        }
        project::ProjectKind::Library => {
            match crate::project_build::check_library(&loaded, standard_library) {
                Ok(()) => 0,
                Err(message) => {
                    let _ = writeln!(stderr, "{message}");
                    1
                }
            }
        }
        project::ProjectKind::Test => check_test_project(&loaded, standard_library, stderr),
    }
}

fn check_test_project(
    loaded: &project::LoadedProject,
    standard_library: Option<&project::StandardLibrary>,
    stderr: &mut dyn Write,
) -> i32 {
    let unit_files: Vec<PathBuf> = loaded
        .source_files
        .iter()
        .filter(|source| !project::is_test_source_file(source))
        .cloned()
        .collect();

    if !unit_files.is_empty() {
        match crate::project_build::check_units(&unit_files, &loaded.link_meta, standard_library) {
            Ok(()) => {}
            Err(message) => {
                let _ = writeln!(stderr, "{message}");
                return 1;
            }
        }
    }

    for test_path in loaded
        .source_files
        .iter()
        .filter(|source| project::is_test_source_file(source))
    {
        match crate::project_build::build_test_program(
            test_path,
            &unit_files,
            &loaded.link_meta,
            standard_library,
        ) {
            Ok(_) => {}
            Err(message) => {
                let _ = writeln!(stderr, "{message}");
                return 1;
            }
        }
    }

    0
}

fn check_workspace_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    stderr: &mut dyn Write,
) -> i32 {
    let workspace = match project::load_workspace(path) {
        Ok(workspace) => workspace,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 1;
        }
    };

    let mut exit_code = 0;
    for member in &workspace.member_projects {
        let member_exit = check_project_file(member, standard_library, stderr);
        if member_exit != 0 {
            exit_code = member_exit;
        }
    }

    exit_code
}

fn check_parsed_source(
    path: &str,
    source: &str,
    source_paths: Option<&[PathBuf]>,
    stderr: &mut dyn Write,
) -> i32 {
    let (program, parse_errors) = fpas_parser::parse(source);
    let has_errors = parse_errors
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().severity == DiagnosticSeverity::Error);

    for diagnostic in &parse_errors {
        emit_check_diagnostic(path, source_paths, diagnostic.as_diagnostic(), stderr);
    }

    if has_errors {
        return 1;
    }

    check_parsed_program(path, &program, source_paths, stderr)
}

fn check_parsed_program(
    path: &str,
    program: &fpas_parser::Program,
    source_paths: Option<&[PathBuf]>,
    stderr: &mut dyn Write,
) -> i32 {
    match fpas_compiler::compile_all(program) {
        Ok(_chunk) => 0,
        Err(diagnostics) => {
            for diagnostic in &diagnostics {
                emit_check_diagnostic(path, source_paths, diagnostic, stderr);
            }
            1
        }
    }
}

fn emit_check_diagnostic(
    path: &str,
    source_paths: Option<&[PathBuf]>,
    diagnostic: &fpas_diagnostics::Diagnostic,
    stderr: &mut dyn Write,
) {
    let _ = writeln!(
        stderr,
        "{}",
        render_cli_diagnostic_with_sources(path, source_paths, diagnostic)
    );
}
