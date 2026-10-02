//! Run compile and VM from CLI-resolved input.
//!
//! Spec: [Projects & CLI](../../../docs/pascal/program-structure/cli.md);
//! diagnostics: [shared diagnostics](../../../docs/pascal/tools/diagnostics.md).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cli_output::{CliFailure, DiagnosticFormat, Reporter, locate};
use crate::{CliInput, ResolvedCli, resolve_cli_config};
use fpas_diagnostics::codes::{
    BUILD_ARTIFACT_ENCODING_FAILED, BUILD_ARTIFACT_IO_FAILED, CLI_ARGUMENTS_INVALID,
    CLI_INPUT_UNSUPPORTED,
};
use fpas_diagnostics::{DiagnosticSeverity, FileDiagnostic};
use fpas_project as project;

static PROCESS_LIFECYCLE_AUTHORIZED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Grant process authority only from the real binary entry point, never in in-process test hosts.
pub(crate) fn authorize_process_lifecycle() {
    PROCESS_LIFECYCLE_AUTHORIZED.store(true, std::sync::atomic::Ordering::Release);
}

pub(crate) fn run_cli(
    args: &[String],
    cwd: &Path,
    mut stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let resolved = match resolve_cli_config(args, cwd) {
        Ok(resolved) => resolved,
        Err(message) => {
            let mut reporter = Reporter::new(DiagnosticFormat::requested(args), stderr);
            return reporter.failure(&CliFailure::from_message(CLI_ARGUMENTS_INVALID, &message));
        }
    };

    match resolved {
        ResolvedCli::Init(config) => crate::cli_init::init_cli(config, stdout.as_mut(), stderr),
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
        ResolvedCli::Environment => {
            crate::cli_environment::write_environment(stdout.as_mut(), stderr)
        }
        ResolvedCli::Lsp => crate::cli_lsp::run_language_server(cwd, stderr),
        ResolvedCli::Build(config) => {
            let mut reporter = Reporter::new(config.diagnostics, stderr);
            let library = match crate::standard_library::resolve_standard_library(
                config.standard_library.as_deref(),
            ) {
                Ok(library) => library,
                Err(failure) => return reporter.failure(&failure),
            };
            crate::cli_build::build_cli(config, library.as_ref(), stdout.as_mut(), &mut reporter)
        }
        ResolvedCli::Check(config) => {
            let mut reporter = Reporter::new(config.diagnostics, stderr);
            let library = match crate::standard_library::resolve_standard_library(
                config.standard_library.as_deref(),
            ) {
                Ok(library) => library,
                Err(failure) => return reporter.failure(&failure),
            };
            crate::cli_check::check_cli(config, library.as_ref(), &mut reporter)
        }
        ResolvedCli::Fmt(config) => crate::cli_fmt::format_cli(config, stdout.as_mut(), stderr),
        ResolvedCli::Test(config) => {
            let mut reporter = Reporter::new(config.diagnostics, stderr);
            crate::cli_test::test_cli(config, stdout.as_mut(), &mut reporter)
        }
        ResolvedCli::Debug(config) => crate::cli_debug::debug_cli(config, stdout, stderr),
        ResolvedCli::Run(config) => {
            let mut reporter = Reporter::new(config.diagnostics, stderr);
            run_input(config, stdout, &mut reporter)
        }
    }
}

fn run_input(
    config: crate::cli_input::CliConfig,
    stdout: Box<dyn Write + Send>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let input = match config.input {
        CliInput::CompiledProgramFile(path) => {
            return run_compiled_program_file(&path, config.program_args, stdout, reporter);
        }
        input => input,
    };
    let library =
        match crate::standard_library::resolve_standard_library(config.standard_library.as_deref())
        {
            Ok(library) => library,
            Err(failure) => return reporter.failure(&failure),
        };
    match input {
        CliInput::SourceFile(path) if path.is_dir() => reporter.failure(
            &CliFailure::new(
                CLI_INPUT_UNSUPPORTED,
                format!("Cannot run directory `{}`.", path.display()),
            )
            .with_help("Pass a `.fpas` program file or a `.fpasprj` project path."),
        ),
        CliInput::SourceFile(path) => run_source_file(
            &path,
            library.as_ref(),
            config.program_args,
            stdout,
            reporter,
        ),
        CliInput::ProjectFile(path) => run_project_file(
            &path,
            library.as_ref(),
            config.program_args,
            stdout,
            reporter,
        ),
        CliInput::WorkspaceFile(path) => {
            let project_path = match project::discover_run_project_in_workspace(&path) {
                Ok(project_path) => project_path,
                Err(error) => return reporter.failure(&error.into()),
            };
            run_project_file(
                &project_path,
                library.as_ref(),
                config.program_args,
                stdout,
                reporter,
            )
        }
        CliInput::CompiledProgramFile(_) => {
            unreachable!("handled before standard library resolution")
        }
    }
}

fn run_source_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    program_args: Vec<String>,
    stdout: Box<dyn Write + Send>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return reporter.failure(&crate::project_build::source_read_failure(path, &error));
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
            Err(failure) => return reporter.failure(&failure),
        };
        return run_executable(
            path,
            built.executable,
            Some(&built.source_paths),
            program_args,
            stdout,
            reporter,
        );
    }
    run_source_impl(path, &source, program_args, stdout, reporter)
}

fn run_project_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    program_args: Vec<String>,
    stdout: Box<dyn Write + Send>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let loaded = match project::load_project(path) {
        Ok(loaded) => loaded,
        Err(error) => return reporter.failure(&error.into()),
    };
    reporter.records(&loaded.warnings);

    match loaded.kind {
        project::ProjectKind::Program => {
            let artifact =
                match crate::project_build::build_program_artifact(path, &loaded, standard_library)
                {
                    Ok(artifact) => artifact,
                    Err(failure) => return reporter.failure(&failure),
                };
            run_executable(
                &artifact.path,
                artifact.executable,
                Some(&artifact.source_paths),
                program_args,
                stdout,
                reporter,
            )
        }
        project::ProjectKind::Library => reporter.failure(
            &CliFailure::new(
                CLI_INPUT_UNSUPPORTED,
                "Library projects are not executable.",
            )
            .with_help("Use a `program` project to run code with the CLI."),
        ),
        project::ProjectKind::Test => reporter.failure(
            &CliFailure::new(
                CLI_INPUT_UNSUPPORTED,
                "Test projects are not executable with `fpas run`.",
            )
            .with_help(format!(
                "Use `fpas test {}` to run `*_test.fpas` programs.",
                path.display()
            )),
        ),
    }
}

fn run_compiled_program_file(
    path: &Path,
    program_args: Vec<String>,
    stdout: Box<dyn Write + Send>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return reporter.failure(&CliFailure::new(
                BUILD_ARTIFACT_IO_FAILED,
                format!("Cannot read compiled program `{}`: {error}", path.display()),
            ));
        }
    };
    let image = match fpas_program::decode(&bytes) {
        Ok(image) => image,
        Err(error) => {
            return reporter.failure(
                &CliFailure::new(
                    BUILD_ARTIFACT_ENCODING_FAILED,
                    format!("Cannot run compiled program `{}`: {error}", path.display()),
                )
                .with_help("Rebuild the `.fpascp` from its project sources with `fpas build`."),
            );
        }
    };
    let source_paths = image
        .source_paths()
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    run_executable(
        path,
        image.into_executable(),
        Some(&source_paths),
        program_args,
        stdout,
        reporter,
    )
}

fn run_source_impl(
    path: &Path,
    source: &str,
    program_args: Vec<String>,
    stdout: Box<dyn Write + Send>,
    reporter: &mut Reporter<'_>,
) -> i32 {
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

    let executable = match fpas_compiler::compile(&program) {
        Ok(executable) => executable,
        Err(diagnostics) => {
            return reporter.failure(&CliFailure::from_diagnostics(path, &diagnostics));
        }
    };

    run_executable(path, executable, None, program_args, stdout, reporter)
}

#[cfg(test)]
pub(crate) fn run_source(
    path: &str,
    source: &str,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let mut reporter = Reporter::new(DiagnosticFormat::Text, stderr);
    run_source_impl(Path::new(path), source, Vec::new(), stdout, &mut reporter)
}

fn run_executable(
    path: &Path,
    executable: fpas_bytecode::VerifiedExecutable,
    source_paths: Option<&[PathBuf]>,
    program_args: Vec<String>,
    stdout: Box<dyn Write + Send>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    // Runtime diagnostics carry linked source IDs; callers pass paths indexed by graph source.
    let source_paths =
        source_paths.map(|paths| fpas_build::linked_source_paths(&executable, paths));
    let mut vm = fpas_vm::Vm::with_writer_and_args(executable, stdout, program_args);
    if PROCESS_LIFECYCLE_AUTHORIZED.load(std::sync::atomic::Ordering::Acquire) {
        vm.allow_process_lifecycle();
    }
    if let Err(diagnostic) = vm.run() {
        reporter.record(&locate(path, source_paths.as_deref(), &diagnostic));
        return 2;
    }

    0
}

pub(crate) fn render_cli_diagnostic(
    path: &str,
    diagnostic: &fpas_diagnostics::Diagnostic,
) -> String {
    fpas_diagnostics::render(path, diagnostic)
}

#[cfg(test)]
pub(crate) fn render_cli_diagnostic_with_sources(
    fallback_path: &str,
    source_paths: Option<&[PathBuf]>,
    diagnostic: &fpas_diagnostics::Diagnostic,
) -> String {
    locate(Path::new(fallback_path), source_paths, diagnostic).to_string()
}
