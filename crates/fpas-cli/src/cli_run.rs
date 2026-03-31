use std::fs;
use std::io::Write;
use std::path::Path;

use crate::{CliInput, resolve_cli_config};
use fpas_diagnostics::DiagnosticSeverity;
use fpas_lexer::DefineSet;
use fpas_project as project;

pub(crate) fn run_cli(
    args: &[String],
    cwd: &Path,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let config = match resolve_cli_config(args, cwd) {
        Ok(config) => config,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 1;
        }
    };

    match config.input {
        CliInput::SourceFile(path) => run_source_file(&path, &config.defines, stdout, stderr),
        CliInput::ProjectFile(path) => run_project_file(&path, &config.defines, stdout, stderr),
    }
}

fn run_source_file(
    path: &Path,
    defines: &DefineSet,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            let _ = writeln!(stderr, "Error reading `{}`: {error}", path.display());
            return 1;
        }
    };

    let path_text = path.to_string_lossy();
    run_source_with_defines(path_text.as_ref(), &source, defines, stdout, stderr)
}

fn run_project_file(
    path: &Path,
    defines: &DefineSet,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let loaded = match project::load_project_with_defines(path, defines) {
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
            let merged_program =
                match project::build_program_with_defines(&main, &loaded.source_files, defines) {
                    Ok(program) => program,
                    Err(message) => {
                        let _ = writeln!(stderr, "{message}");
                        return 1;
                    }
                };

            let main_path = main.to_string_lossy();
            run_compiled_program(main_path.as_ref(), &merged_program, stdout, stderr)
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

#[cfg(test)]
pub(crate) fn run_source(
    path: &str,
    source: &str,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    run_source_with_defines(path, source, &DefineSet::new(), stdout, stderr)
}

fn run_source_with_defines(
    path: &str,
    source: &str,
    defines: &DefineSet,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let (program, parse_errors) = fpas_parser::parse_with_defines(source, defines);
    let has_errors = parse_errors
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().severity == DiagnosticSeverity::Error);

    for diagnostic in &parse_errors {
        if !emit_diagnostic(path, diagnostic.as_diagnostic(), stderr) {
            return 1;
        }
    }

    if has_errors {
        return 1;
    }

    run_compiled_program(path, &program, stdout, stderr)
}

fn run_compiled_program(
    path: &str,
    program: &fpas_parser::Program,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let chunk = match fpas_compiler::compile(program) {
        Ok(chunk) => chunk,
        Err(diagnostic) => {
            if !emit_diagnostic(path, &diagnostic, stderr) {
                return 1;
            }
            return 1;
        }
    };

    let mut vm = fpas_vm::Vm::with_writer(chunk, stdout);
    if let Err(diagnostic) = vm.run() {
        if !emit_diagnostic(path, &diagnostic, stderr) {
            return 2;
        }
        return 2;
    }

    0
}

fn emit_diagnostic(
    path: &str,
    diagnostic: &fpas_diagnostics::Diagnostic,
    stderr: &mut dyn Write,
) -> bool {
    writeln!(stderr, "{}", render_cli_diagnostic(path, diagnostic)).is_ok()
}

pub(crate) fn render_cli_diagnostic(
    path: &str,
    diagnostic: &fpas_diagnostics::Diagnostic,
) -> String {
    fpas_diagnostics::render(path, diagnostic)
}
