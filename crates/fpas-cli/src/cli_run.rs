//! Run compile and VM from CLI-resolved input.
//!
//! Spec: [Projects & CLI](../../../docs/pascal/program-structure/cli.md).

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
    let resolved = match resolve_cli_config(args, cwd) {
        Ok(resolved) => resolved,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 1;
        }
    };

    match resolved {
        ResolvedCli::Help => {
            use crate::cli_input::CLI_HELP;
            let _ = stdout.write_all(CLI_HELP.as_bytes());
            0
        }
        ResolvedCli::Version => {
            let _ = writeln!(stdout, "fpas {}", env!("CARGO_PKG_VERSION"));
            0
        }
        ResolvedCli::Check(config) => {
            let library = match crate::standard_library::resolve_standard_library(
                config.standard_library.as_deref(),
            ) {
                Ok(library) => library,
                Err(message) => {
                    let _ = writeln!(stderr, "{message}");
                    return 1;
                }
            };
            crate::cli_check::check_cli(config, library.as_ref(), stderr)
        }
        ResolvedCli::Fmt(config) => crate::cli_fmt::format_cli(config, stdout.as_mut(), stderr),
        ResolvedCli::Test(config) => crate::cli_test::test_cli(config, stdout.as_mut(), stderr),
        ResolvedCli::Run(config) => {
            let library = match crate::standard_library::resolve_standard_library(
                config.standard_library.as_deref(),
            ) {
                Ok(library) => library,
                Err(message) => {
                    let _ = writeln!(stderr, "{message}");
                    return 1;
                }
            };
            match config.input {
                CliInput::SourceFile(path) if path.is_dir() => {
                    let _ = writeln!(
                        stderr,
                        "Cannot run directory `{}`.\n  help: Pass a `.fpas` program file or a `.fpasprj` project path.",
                        path.display()
                    );
                    1
                }
                CliInput::SourceFile(path) => {
                    run_source_file(&path, library.as_ref(), config.program_args, stdout, stderr)
                }
                CliInput::ProjectFile(path) => {
                    run_project_file(&path, library.as_ref(), config.program_args, stdout, stderr)
                }
                CliInput::WorkspaceFile(path) => {
                    let _ = writeln!(
                        stderr,
                        "Cannot run workspace `{}`.\n  help: Use `fpas check` to validate workspace members, or pass a `.fpasprj` program path.",
                        path.display()
                    );
                    1
                }
            }
        }
    }
}

fn run_source_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    program_args: Vec<String>,
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

    if let Some(standard_library) = standard_library {
        let linked = match project::build_program_with_standard_library(
            path,
            &[],
            &project::ProjectLinkMeta::default(),
            standard_library,
        ) {
            Ok(linked) => linked,
            Err(message) => {
                let _ = writeln!(stderr, "{message}");
                return 1;
            }
        };
        let path_text = path.to_string_lossy();
        return run_compiled_program(
            path_text.as_ref(),
            &linked.program,
            Some(&linked.source_paths),
            program_args,
            stdout,
            stderr,
        );
    }
    let path_text = path.to_string_lossy();
    run_source_impl(path_text.as_ref(), &source, program_args, stdout, stderr)
}

fn run_project_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    program_args: Vec<String>,
    stdout: Box<dyn Write + Send>,
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
            let Some(main) = loaded.main else {
                let _ = writeln!(
                    stderr,
                    "Project is missing `project.main`.\n  help: Set `main = \"src/main.fpas\"` in `[project]`."
                );
                return 1;
            };
            let linked_program = match standard_library.map_or_else(
                || {
                    project::build_program_with_source_map(
                        &main,
                        &loaded.source_files,
                        &loaded.link_meta,
                    )
                },
                |library| {
                    project::build_program_with_standard_library(
                        &main,
                        &loaded.source_files,
                        &loaded.link_meta,
                        library,
                    )
                },
            ) {
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
                program_args,
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
        project::ProjectKind::Test => {
            let _ = writeln!(
                stderr,
                "Test projects are not executable with `fpas run`.\n  help: Use `fpas test {}` to run `*_test.fpas` programs.",
                path.display()
            );
            1
        }
    }
}

fn run_source_impl(
    path: &str,
    source: &str,
    program_args: Vec<String>,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let (program, parse_errors) = fpas_parser::parse(source);
    let has_errors = parse_errors
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().severity == DiagnosticSeverity::Error);

    for diagnostic in &parse_errors {
        emit_diagnostic(path, None, diagnostic.as_diagnostic(), stderr);
    }

    if has_errors {
        return 1;
    }

    run_compiled_program(path, &program, None, program_args, stdout, stderr)
}

#[cfg(test)]
pub(crate) fn run_source(
    path: &str,
    source: &str,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    run_source_impl(path, source, Vec::new(), stdout, stderr)
}

fn run_compiled_program(
    path: &str,
    program: &fpas_parser::Program,
    source_paths: Option<&[PathBuf]>,
    program_args: Vec<String>,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let chunk = match fpas_compiler::compile_all(program) {
        Ok(chunk) => chunk,
        Err(diagnostics) => {
            for diagnostic in &diagnostics {
                emit_diagnostic(path, source_paths, diagnostic, stderr);
            }
            return 1;
        }
    };

    let mut vm = fpas_vm::Vm::with_writer_and_args(chunk, stdout, program_args);
    if let Err(diagnostic) = vm.run() {
        emit_diagnostic(path, source_paths, &diagnostic, stderr);
        return 2;
    }

    0
}

fn emit_diagnostic(
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

pub(crate) fn render_cli_diagnostic(
    path: &str,
    diagnostic: &fpas_diagnostics::Diagnostic,
) -> String {
    fpas_diagnostics::render(path, diagnostic)
}

pub(crate) fn render_cli_diagnostic_with_sources(
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
