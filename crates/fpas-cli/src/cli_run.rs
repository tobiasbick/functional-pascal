//! Run compile and VM from CLI-resolved input.
//!
//! Spec: [Projects & CLI](../../../docs/pascal/10-projects.md).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{CliInput, ResolvedCli, resolve_cli_config};
use fpas_diagnostics::DiagnosticSeverity;
use fpas_project as project;

pub(crate) fn run_cli(
    args: &[String],
    cwd: &Path,
    mut stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let config = match resolve_cli_config(args, cwd) {
        Ok(ResolvedCli::Run(config)) => config,
        Ok(ResolvedCli::Help) => {
            use crate::cli_input::CLI_HELP;
            let _ = stdout.write_all(CLI_HELP.as_bytes());
            return 0;
        }
        Ok(ResolvedCli::Version) => {
            let _ = writeln!(stdout, "fpas {}", env!("CARGO_PKG_VERSION"));
            return 0;
        }
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 1;
        }
    };

    match config.input {
        CliInput::SourceFile(path) => run_source_file(&path, stdout, stderr),
        CliInput::ProjectFile(path) => run_project_file(&path, stdout, stderr),
    }
}

fn run_source_file(path: &Path, stdout: Box<dyn Write + Send>, stderr: &mut dyn Write) -> i32 {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            let _ = writeln!(stderr, "Error reading `{}`: {error}", path.display());
            return 1;
        }
    };

    let path_text = path.to_string_lossy();
    run_source_impl(path_text.as_ref(), &source, stdout, stderr)
}

fn run_project_file(path: &Path, stdout: Box<dyn Write + Send>, stderr: &mut dyn Write) -> i32 {
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
            let linked_program =
                match project::build_program_with_source_map(&main, &loaded.source_files) {
                    Ok(program) => program,
                    Err(message) => {
                        let _ = writeln!(stderr, "{message}");
                        return 1;
                    }
                };

            let main_path = main.to_string_lossy();
            run_compiled_program(
                main_path.as_ref(),
                &linked_program.program,
                Some(&linked_program.source_paths),
                stdout,
                stderr,
            )
        }
        project::ProjectKind::Library => {
            let _ = writeln!(
                stderr,
                "Library projects are not executable.\n  help: Use a `program` project to run code with the CLI."
            );
            1
        }
    }
}

fn run_source_impl(
    path: &str,
    source: &str,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let (program, parse_errors) = fpas_parser::parse(source);
    let has_errors = parse_errors
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().severity == DiagnosticSeverity::Error);

    for diagnostic in &parse_errors {
        if !emit_diagnostic(path, None, diagnostic.as_diagnostic(), stderr) {
            return 1;
        }
    }

    if has_errors {
        return 1;
    }

    run_compiled_program(path, &program, None, stdout, stderr)
}

#[cfg(test)]
pub(crate) fn run_source(
    path: &str,
    source: &str,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    run_source_impl(path, source, stdout, stderr)
}

fn run_compiled_program(
    path: &str,
    program: &fpas_parser::Program,
    source_paths: Option<&[PathBuf]>,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let chunk = match fpas_compiler::compile_all(program) {
        Ok(chunk) => chunk,
        Err(diagnostics) => {
            for diagnostic in &diagnostics {
                if !emit_diagnostic(path, source_paths, diagnostic, stderr) {
                    return 1;
                }
            }
            return 1;
        }
    };

    let mut vm = fpas_vm::Vm::with_writer(chunk, stdout);
    if let Err(diagnostic) = vm.run() {
        if !emit_diagnostic(path, source_paths, &diagnostic, stderr) {
            return 2;
        }
        return 2;
    }

    0
}

fn emit_diagnostic(
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

pub(crate) fn render_cli_diagnostic(
    path: &str,
    diagnostic: &fpas_diagnostics::Diagnostic,
) -> String {
    fpas_diagnostics::render(path, diagnostic)
}

fn render_cli_diagnostic_with_sources(
    fallback_path: &str,
    source_paths: Option<&[PathBuf]>,
    diagnostic: &fpas_diagnostics::Diagnostic,
) -> String {
    let Some(path) = source_paths
        .and_then(|paths| {
            usize::try_from(diagnostic.span.source_id)
                .ok()
                .and_then(|index| paths.get(index))
        })
        .map(|path| path.to_string_lossy().into_owned())
    else {
        return render_cli_diagnostic(fallback_path, diagnostic);
    };

    fpas_diagnostics::render(&path, diagnostic)
}
