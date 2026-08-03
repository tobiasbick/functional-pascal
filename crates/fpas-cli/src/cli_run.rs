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
        ResolvedCli::Help(topic) => {
            match crate::cli_output::write_stdout(
                stdout.as_mut(),
                stderr,
                "help to stdout",
                |stdout| stdout.write_all(crate::cli_input::help_text(topic).as_bytes()),
            ) {
                Ok(()) => 0,
                Err(exit_code) => exit_code,
            }
        }
        ResolvedCli::Version => {
            match crate::cli_output::write_stdout(
                stdout.as_mut(),
                stderr,
                "version to stdout",
                |stdout| writeln!(stdout, "fpas {}", env!("CARGO_PKG_VERSION")),
            ) {
                Ok(()) => 0,
                Err(exit_code) => exit_code,
            }
        }
        ResolvedCli::Build(config) => {
            let library = match crate::standard_library::resolve_standard_library(
                config.standard_library.as_deref(),
            ) {
                Ok(library) => library,
                Err(message) => {
                    let _ = writeln!(stderr, "{message}");
                    return 1;
                }
            };
            crate::cli_build::build_cli(config, library.as_ref(), stdout.as_mut(), stderr)
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
            let input = match config.input {
                CliInput::CompiledProgramFile(path) => {
                    return run_compiled_program_file(&path, config.program_args, stdout, stderr);
                }
                input => input,
            };
            let library = match crate::standard_library::resolve_standard_library(
                config.standard_library.as_deref(),
            ) {
                Ok(library) => library,
                Err(message) => {
                    let _ = writeln!(stderr, "{message}");
                    return 1;
                }
            };
            match input {
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
                    run_workspace_file(&path, library.as_ref(), config.program_args, stdout, stderr)
                }
                CliInput::CompiledProgramFile(_) => {
                    unreachable!("handled before standard library resolution")
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
        let built = match crate::project_build::build_test_program(
            path,
            &[],
            &project::ProjectLinkMeta::default(),
            Some(standard_library),
        ) {
            Ok(built) => built,
            Err(message) => {
                let _ = writeln!(stderr, "{message}");
                return 1;
            }
        };
        let path_text = path.to_string_lossy();
        return run_chunk(
            path_text.as_ref(),
            built.chunk,
            Some(&built.source_paths),
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
            let artifact =
                match crate::project_build::build_program_artifact(path, &loaded, standard_library)
                {
                    Ok(artifact) => artifact,
                    Err(message) => {
                        let _ = writeln!(stderr, "{message}");
                        return 1;
                    }
                };

            let artifact_path = artifact.path.to_string_lossy();
            run_chunk(
                artifact_path.as_ref(),
                artifact.chunk,
                Some(&artifact.source_paths),
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

fn run_workspace_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    program_args: Vec<String>,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let project_path = match project::discover_run_project_in_workspace(path) {
        Ok(project_path) => project_path,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 1;
        }
    };
    run_project_file(
        &project_path,
        standard_library,
        program_args,
        stdout,
        stderr,
    )
}

fn run_compiled_program_file(
    path: &Path,
    program_args: Vec<String>,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            let _ = writeln!(
                stderr,
                "Cannot read compiled program `{}`: {error}",
                path.display()
            );
            return 1;
        }
    };
    let image = match fpas_program::decode(&bytes) {
        Ok(image) => image,
        Err(error) => {
            let _ = writeln!(
                stderr,
                "Cannot run compiled program `{}`: {error}\n  help: Rebuild the `.fpascp` from its project sources with `fpas build`.",
                path.display()
            );
            return 1;
        }
    };
    let source_paths = image
        .source_paths()
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let path_text = path.to_string_lossy();
    run_chunk(
        path_text.as_ref(),
        image.into_chunk(),
        Some(&source_paths),
        program_args,
        stdout,
        stderr,
    )
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

    run_chunk(path, chunk, source_paths, program_args, stdout, stderr)
}

fn run_chunk(
    path: &str,
    chunk: fpas_bytecode::Chunk,
    source_paths: Option<&[PathBuf]>,
    program_args: Vec<String>,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
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
