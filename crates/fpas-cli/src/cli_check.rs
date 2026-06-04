//! Type-check projects and workspaces without running the VM.
//!
//! Documentation: `docs/pascal/10-projects.md`

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::CliInput;
use crate::cli_run::render_cli_diagnostic_with_sources;
use fpas_diagnostics::DiagnosticSeverity;
use fpas_project as project;

/// Checks sources from CLI-resolved input without execution.
pub(crate) fn check_cli(config: crate::CliConfig, stderr: &mut dyn Write) -> i32 {
    match config.input {
        CliInput::SourceFile(path) => check_source_file(&path, stderr),
        CliInput::ProjectFile(path) => check_project_file(&path, stderr),
        CliInput::WorkspaceFile(path) => check_workspace_file(&path, stderr),
    }
}

fn check_source_file(path: &Path, stderr: &mut dyn Write) -> i32 {
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

fn check_project_file(path: &Path, stderr: &mut dyn Write) -> i32 {
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
            let Some(main) = loaded.main else {
                let _ = writeln!(
                    stderr,
                    "Project is missing `project.main`.\n  help: Set `main = \"src/main.fpas\"` in `[project]`."
                );
                return 1;
            };
            let linked = match project::build_program_with_source_map(
                &main,
                &loaded.source_files,
                &loaded.link_meta,
            ) {
                Ok(program) => program,
                Err(message) => {
                    let _ = writeln!(stderr, "{message}");
                    return 1;
                }
            };
            let main_path = main.to_string_lossy();
            check_parsed_program(
                main_path.as_ref(),
                &linked.program,
                Some(&linked.source_paths),
                stderr,
            )
        }
        project::ProjectKind::Library => {
            let linked = match project::build_library_check_with_source_map(
                &loaded.source_files,
                &loaded.link_meta,
            ) {
                Ok(program) => program,
                Err(message) => {
                    let _ = writeln!(stderr, "{message}");
                    return 1;
                }
            };
            let path_text = path.to_string_lossy();
            check_parsed_program(
                path_text.as_ref(),
                &linked.program,
                Some(&linked.source_paths),
                stderr,
            )
        }
    }
}

fn check_workspace_file(path: &Path, stderr: &mut dyn Write) -> i32 {
    let workspace = match project::load_workspace(path) {
        Ok(workspace) => workspace,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 1;
        }
    };

    let mut exit_code = 0;
    for member in &workspace.member_projects {
        let member_exit = check_project_file(member, stderr);
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
        if !emit_check_diagnostic(path, source_paths, diagnostic.as_diagnostic(), stderr) {
            return 1;
        }
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
                if !emit_check_diagnostic(path, source_paths, diagnostic, stderr) {
                    return 1;
                }
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
) -> bool {
    writeln!(
        stderr,
        "{}",
        render_cli_diagnostic_with_sources(path, source_paths, diagnostic)
    )
    .is_ok()
}
