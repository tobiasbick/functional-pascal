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

#[derive(Default)]
struct ExecutionOptions {
    no_contracts: bool,
    program_args: Vec<String>,
    failure_dump: Option<PathBuf>,
    profile: Option<PathBuf>,
    /// Input kind and path recorded by profile reports.
    target: (&'static str, PathBuf),
    collection_schedule: fpas_bytecode::CollectionSchedule,
}

static PROCESS_LIFECYCLE_AUTHORIZED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Grant process authority only from the real binary entry point, never in in-process test hosts.
pub(crate) fn authorize_process_lifecycle() {
    PROCESS_LIFECYCLE_AUTHORIZED.store(true, std::sync::atomic::Ordering::Release);
}

/// Resolve and execute one command while preserving its stdout/diagnostic contracts.
pub(crate) fn run_cli(
    args: &[String],
    cwd: &Path,
    mut stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
    color: bool,
) -> i32 {
    if crate::cli_query::selected(args) {
        return crate::cli_query::run(args, cwd, stdout.as_mut(), stderr);
    }
    if crate::cli_batch::selected(args) {
        return crate::cli_batch::run(args, cwd, stdout.as_mut(), stderr);
    }
    let resolved = match resolve_cli_config(args, cwd) {
        Ok(resolved) => resolved,
        Err(message) => {
            let mut reporter =
                Reporter::new(DiagnosticFormat::requested(args), stderr).with_color(color);
            return reporter.failure(&CliFailure::from_message(CLI_ARGUMENTS_INVALID, &message));
        }
    };

    match resolved {
        ResolvedCli::StageStdlib(config) => {
            let mut reporter = Reporter::new(DiagnosticFormat::Text, stderr).with_color(color);
            crate::cli_stdlib::stage(config, stdout.as_mut(), &mut reporter)
        }
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
            let mut reporter = Reporter::new(config.diagnostics, stderr).with_color(color);
            let library = match crate::standard_library::resolve_standard_library(
                config.standard_library.as_deref(),
            ) {
                Ok(library) => library,
                Err(failure) => return reporter.failure(&failure),
            };
            crate::cli_build::build_cli(config, library.as_ref(), stdout.as_mut(), &mut reporter)
        }
        ResolvedCli::Check(config) => {
            let mut reporter = Reporter::new(config.diagnostics, stderr).with_color(color);
            let library = match crate::standard_library::resolve_standard_library(
                config.standard_library.as_deref(),
            ) {
                Ok(library) => library,
                Err(failure) => return reporter.failure(&failure),
            };
            crate::cli_check::check_cli(config, library.as_ref(), &mut reporter)
        }
        ResolvedCli::Doc(config) => {
            let mut reporter = Reporter::new(DiagnosticFormat::Text, stderr).with_color(color);
            crate::cli_doc::doc_cli(&config, stdout.as_mut(), &mut reporter)
        }
        ResolvedCli::Query(config) => {
            let mut reporter = Reporter::new(DiagnosticFormat::Json, stderr);
            crate::cli_query::query_cli(&config, stdout.as_mut(), &mut reporter)
        }
        ResolvedCli::Fmt(config) => crate::cli_fmt::format_cli(config, stdout.as_mut(), stderr),
        ResolvedCli::Test(config) => {
            let mut reporter = Reporter::new(config.diagnostics, stderr).with_color(color);
            crate::cli_test::test_cli(config, stdout.as_mut(), &mut reporter)
        }
        ResolvedCli::Debug(config) => crate::cli_debug::debug_cli(*config, stdout, stderr),
        ResolvedCli::Run(config) => {
            let mut reporter = Reporter::new(config.diagnostics, stderr).with_color(color);
            run_input(config, stdout, &mut reporter)
        }
    }
}

fn run_input(
    config: crate::cli_input::CliConfig,
    stdout: Box<dyn Write + Send>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let target = match &config.input {
        CliInput::SourceFile(path) => ("source", path.clone()),
        CliInput::ProjectFile(path) => ("project", path.clone()),
        CliInput::WorkspaceFile(path) => ("workspace", path.clone()),
        CliInput::CompiledProgramFile(path) => ("compiled", path.clone()),
    };
    let options = ExecutionOptions {
        no_contracts: config.no_contracts,
        program_args: config.program_args,
        failure_dump: config.failure_dump,
        profile: config.profile,
        target,
        collection_schedule: config.collection_schedule,
    };
    let input = match config.input {
        CliInput::CompiledProgramFile(path) => {
            return run_compiled_program_file(&path, options, stdout, reporter);
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
        CliInput::SourceFile(path) => {
            run_source_file(&path, library.as_ref(), options, stdout, reporter)
        }
        CliInput::ProjectFile(path) => {
            run_project_file(&path, library.as_ref(), options, stdout, reporter)
        }
        CliInput::WorkspaceFile(path) => {
            let project_path = match project::discover_run_project_in_workspace(&path) {
                Ok(project_path) => project_path,
                Err(error) => return reporter.failure(&error.into()),
            };
            run_project_file(&project_path, library.as_ref(), options, stdout, reporter)
        }
        CliInput::CompiledProgramFile(_) => {
            unreachable!("handled before standard library resolution")
        }
    }
}

fn run_source_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    options: ExecutionOptions,
    stdout: Box<dyn Write + Send>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return reporter.failure(&crate::project_build::source_read_failure(path, &error));
        }
    };
    reporter.remember_source(path, &source);

    if let Some(standard_library) = standard_library {
        let built = match crate::project_build::build_test_program(
            path,
            &[],
            &project::ProjectLinkMeta::default(),
            Some(standard_library),
            options.no_contracts,
        ) {
            Ok(built) => built,
            Err(failure) => return reporter.failure(&failure),
        };
        reporter.load_sources(&built.sources);
        return run_executable(
            path,
            built.executable,
            Some(&built.source_paths),
            options,
            stdout,
            reporter,
        );
    }
    run_source_impl(path, &source, options, stdout, reporter)
}

fn run_project_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    options: ExecutionOptions,
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
            let artifact = match crate::project_build::build_program_artifact(
                path,
                &loaded,
                standard_library,
                options.no_contracts,
            ) {
                Ok(artifact) => artifact,
                Err(failure) => return reporter.failure(&failure),
            };
            reporter.load_sources(&artifact.sources);
            run_executable(
                &artifact.path,
                artifact.executable,
                Some(&artifact.source_paths),
                options,
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
    options: ExecutionOptions,
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
    if options.no_contracts && image.identity().contracts_enabled {
        return reporter.failure(&CliFailure::new(CLI_INPUT_UNSUPPORTED,
            "The compiled program contains contracts; `--no-contracts` cannot remove code from an existing image.")
            .with_help("Rebuild its source project with `fpas build --no-contracts project.fpasprj`, then run the new image."));
    }
    // Portable image labels are not source roots. Only exact paths with matching hashes qualify.
    reporter.load_sources(
        &source_paths
            .iter()
            .zip(image.source_hashes())
            .map(|(path, hash)| crate::cli_output::RuntimeSource::hashed(path, *hash))
            .collect::<Vec<_>>(),
    );
    run_executable(
        path,
        image.into_executable(),
        Some(&source_paths),
        options,
        stdout,
        reporter,
    )
}

fn run_source_impl(
    path: &Path,
    source: &str,
    options: ExecutionOptions,
    stdout: Box<dyn Write + Send>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    reporter.remember_source(path, source);
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

    let executable = match fpas_compiler::compile_with_options(
        &program,
        fpas_compiler::CompileOptions {
            contracts: !options.no_contracts,
        },
    ) {
        Ok(executable) => executable,
        Err(diagnostics) => {
            return reporter.failure(&CliFailure::from_diagnostics(path, &diagnostics));
        }
    };

    run_executable(path, executable, None, options, stdout, reporter)
}

#[cfg(test)]
/// Execute a source fixture through ordinary runtime diagnostic rendering.
pub(crate) fn run_source(
    path: &str,
    source: &str,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let mut reporter = Reporter::new(DiagnosticFormat::Text, stderr);
    run_source_impl(
        Path::new(path),
        source,
        ExecutionOptions::default(),
        stdout,
        &mut reporter,
    )
}

fn run_executable(
    path: &Path,
    executable: fpas_bytecode::VerifiedExecutable,
    source_paths: Option<&[PathBuf]>,
    options: ExecutionOptions,
    stdout: Box<dyn Write + Send>,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let ExecutionOptions {
        no_contracts: _,
        program_args,
        failure_dump,
        profile,
        target,
        collection_schedule,
    } = options;
    let profile = profile.map(|destination| {
        let target = crate::cli_profile::Target::new(target.0, &target.1, executable.executable());
        (destination, target)
    });
    // Runtime diagnostics carry linked source IDs; callers pass paths indexed by graph source.
    let source_paths =
        source_paths.map(|paths| fpas_build::linked_source_paths(&executable, paths));
    let sources = failure_dump
        .as_ref()
        .map(|_| crate::cli_failure_dump::Sources::new(&executable, source_paths.as_deref()));
    let mut vm = fpas_vm::Vm::with_writer_and_args(executable, stdout, program_args)
        .with_collection_schedule(collection_schedule);
    if let Some(terminal) = fpas_vm::InteractiveProcessTerminal::inherited() {
        vm.set_interactive_process_terminal(terminal);
    }
    let session = failure_dump
        .clone()
        .zip(sources)
        .map(|(path, sources)| crate::cli_failure_dump::Session::start(&mut vm, path, sources));
    let profile = profile.map(|(destination, target)| {
        crate::cli_profile::Session::start(&mut vm, destination, target, collection_schedule)
    });
    if PROCESS_LIFECYCLE_AUTHORIZED.load(std::sync::atomic::Ordering::Acquire) {
        vm.allow_process_lifecycle();
    }
    if let Some(receiver) = crate::cli_output::program_stderr_receiver(reporter.format()) {
        vm.set_program_stderr(receiver);
    }
    let outcome = vm.run();
    let publication = session.map(|session| {
        session
            .and_then(|session| session.finish(outcome.as_ref().err().map(|error| error.as_ref())))
    });
    let exit_code = if outcome.is_err() { 2 } else { 0 };
    if let Some(profile) = profile {
        let destination = profile.destination().to_path_buf();
        if let Err(error) = profile.finish(&vm, exit_code, source_paths.as_deref()) {
            let failure = CliFailure::new(
                fpas_diagnostics::codes::CLI_OUTPUT_FAILED,
                format!(
                    "Cannot publish profile `{}`: {error}",
                    destination.display()
                ),
            )
            .with_help("Choose a new writable file path.");
            let code = reporter.failure(&failure);
            if exit_code == 0 {
                return code;
            }
        }
    }
    if let Err(diagnostic) = outcome {
        reporter.record(&locate(path, source_paths.as_deref(), &diagnostic));
        if let Some(Err(error)) = publication {
            let failure = CliFailure::new(fpas_diagnostics::codes::CLI_OUTPUT_FAILED,
                format!("Cannot publish failure dump `{}`: {error}", failure_dump.as_deref().unwrap_or(Path::new("<report>")).display()))
                .with_help("Choose a new writable file path on a filesystem supporting atomic hard-link publication.");
            reporter.failure(&failure);
        }
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
