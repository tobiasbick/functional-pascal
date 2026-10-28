//! CLI argument resolution and project discovery.
//!
//! Spec: [Projects & CLI](../../../docs/pascal/program-structure/cli.md).

mod batch;
mod collection;
pub(crate) use batch::parse_seed as parse_schedule_seed;
mod discovery;
mod doc;
mod help;
mod init;
mod mode;
mod options;
mod query;
mod stdlib;
mod types;
pub(crate) use stdlib::StdlibCliConfig;

use std::path::Path;
use std::time::Duration;

const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(300);

pub(crate) use discovery::discover_check_input;
pub(crate) use help::help_text;
pub(crate) use query::{QueryCliConfig, QueryKind};
pub(crate) use types::{
    BuildCliConfig, CliConfig, CliInput, DebugCliConfig, DebugProtocol, DocCliConfig,
    DocReportFormat, FmtCliConfig, HelpTopic, InitCliConfig, InitKind, InitReportFormat,
    ResolvedCli, TestCliConfig, TestReportFormat,
};

use init::resolve_init_cli;
use mode::{
    CliMode, parse_cli_mode, program_args_require_run_error, split_program_args, usage_error,
};
use options::parse_options;

use discovery::{discover_input, resolve_explicit_input};

/// Resolves at most one positional path or project discovery, for unit tests only.
///
/// [`resolve_cli_config`] is the full CLI entry: it also handles `--help` / `--version`.
#[cfg(test)]
pub(crate) fn resolve_cli_input(args: &[String], cwd: &Path) -> Result<CliInput, String> {
    match resolve_cli_config(args, cwd)? {
        ResolvedCli::Init(_) => {
            Err("resolve_cli_input: use resolve_cli_config for `fpas init`".to_string())
        }
        ResolvedCli::Build(config) => Ok(config.input),
        ResolvedCli::Run(config) | ResolvedCli::Check(config) => Ok(config.input),
        ResolvedCli::Debug(config) => Ok(config.input),
        ResolvedCli::Fmt(_) => {
            Err("resolve_cli_input: use resolve_cli_config for `fpas fmt`".to_string())
        }
        ResolvedCli::Test(_)
        | ResolvedCli::Doc(_)
        | ResolvedCli::Query(_)
        | ResolvedCli::StageStdlib(_)
        | ResolvedCli::Environment
        | ResolvedCli::Help(_)
        | ResolvedCli::Version => {
            Err("resolve_cli_input: use resolve_cli_config for --help or --version".to_string())
        }
    }
}

pub(crate) fn resolve_cli_config(args: &[String], cwd: &Path) -> Result<ResolvedCli, String> {
    let (cli_args, program_args) = split_program_args(args);

    if cli_args.is_empty() {
        if !program_args.is_empty() {
            return Err(program_args_require_run_error());
        }
        return Ok(ResolvedCli::Help(HelpTopic::General));
    }

    if cli_args.len() == 1 {
        match cli_args[0].as_str() {
            "-h" | "--help" => return Ok(ResolvedCli::Help(HelpTopic::General)),
            "-V" | "--version" => return Ok(ResolvedCli::Version),
            _ => {}
        }
    }

    if cli_args[0] == "stdlib" {
        if !program_args.is_empty() {
            return Err("`fpas stdlib` does not accept program arguments.\n  help: `fpas stdlib stage --help`.".into());
        }
        return stdlib::resolve(&cli_args[1..], cwd);
    }

    if cli_args[0] == "query" {
        if !program_args.is_empty() {
            return Err("`fpas query` does not accept program arguments.\n  help: `fpas query refs src/main.fpas:12:5`.".into());
        }
        return query::resolve(&cli_args[1..], cwd);
    }

    let (mode, cli_args) = parse_cli_mode(cli_args)?;

    if mode == CliMode::Init {
        if !program_args.is_empty() {
            return Err(
                "`fpas init` does not accept arguments after `--`.\n  help: `fpas init --help` shows valid scaffold commands."
                    .to_string(),
            );
        }
        return resolve_init_cli(cli_args, cwd);
    }

    if mode == CliMode::Doc {
        if !program_args.is_empty() {
            return Err("`fpas doc` does not accept arguments after `--`.
  help: `fpas doc --help` shows valid usage."
                .to_string());
        }
        return doc::resolve_doc_cli(cli_args, cwd);
    }

    if mode == CliMode::Env {
        if !program_args.is_empty() {
            return Err(format!(
                "`fpas {}` does not accept arguments after `--`.\n  help: `fpas {} --help` shows valid usage.",
                mode.name(),
                mode.name()
            ));
        }
        return resolve_environment_command(cli_args);
    }

    if !matches!(mode, CliMode::Run | CliMode::Debug) && !program_args.is_empty() {
        let cmd = match mode {
            CliMode::Init | CliMode::Doc => {
                unreachable!("init and doc are handled before shared option parsing")
            }
            CliMode::Build => "fpas build",
            CliMode::Check => "fpas check",
            CliMode::Fmt => "fpas fmt",
            CliMode::Test => "fpas test",
            CliMode::Run => unreachable!(),
            CliMode::Debug => unreachable!(),
            CliMode::Env => unreachable!("environment command handled above"),
        };
        return Err(format!(
            "`{cmd}` does not accept program arguments after `--`.\n  help: Omit `--` and trailing program arguments."
        ));
    }

    let options = parse_options(mode, cli_args)?;
    let diagnostics = options.diagnostics.unwrap_or_default();
    let report_destination = |path: std::path::PathBuf, what: &str| {
        let path = if path.is_absolute() {
            path
        } else {
            cwd.join(path)
        };
        crate::cli_failure_dump::validate_destination(&path, what)?;
        Ok::<_, String>(path)
    };
    let failure_dump = options
        .failure_dump
        .map(|path| report_destination(path, "Failure dump"))
        .transpose()?;
    let profile = options
        .profile
        .map(|path| report_destination(path, "Profile"))
        .transpose()?;
    let positional = options.positional;

    if mode == CliMode::Fmt {
        if options.stdout_mode && options.check_only {
            return Err(
                "`fpas fmt --stdout` cannot be combined with `--check`.\n  help: Use `--stdout` to print formatted output, or `--check` to verify on disk."
                    .to_string(),
            );
        }
        if options.list_changed && !options.check_only {
            return Err(
                "`fpas fmt --list` requires `--check`.\n  help: `fpas fmt --check --list <path>...` prints paths that would change."
                    .to_string(),
            );
        }
        for arg in &positional {
            if arg == "-h" || arg == "--help" {
                if positional.len() != 1 {
                    return Err(usage_error(mode));
                }
                return Ok(ResolvedCli::Help(mode.help_topic()));
            }
            if arg == "-V" || arg == "--version" {
                if positional.len() != 1 {
                    return Err(usage_error(mode));
                }
                return Ok(ResolvedCli::Version);
            }
            if arg.starts_with('-') {
                return Err(format!(
                    "Unknown option `{arg}`.\n  help: Pass one or more paths, or `fpas fmt --help`."
                ));
            }
        }
        return Ok(ResolvedCli::Fmt(FmtCliConfig {
            explicit_args: positional,
            cwd: cwd.to_path_buf(),
            check_only: options.check_only,
            stdout: options.stdout_mode,
            list_changed: options.list_changed,
        }));
    }

    let mut input = None::<String>;
    for arg in &positional {
        if arg == "-h" || arg == "--help" {
            if positional.len() != 1 {
                return Err(usage_error(mode));
            }
            return Ok(ResolvedCli::Help(mode.help_topic()));
        }
        if arg == "-V" || arg == "--version" {
            if positional.len() != 1 {
                return Err(usage_error(mode));
            }
            return Ok(ResolvedCli::Version);
        }

        if arg.starts_with('-') {
            let hint = match mode {
                CliMode::Init | CliMode::Doc => {
                    unreachable!("init and doc are handled before shared option parsing")
                }
                CliMode::Build => {
                    "Pass a project or workspace path after `fpas build`, or use `fpas build --help`."
                }
                CliMode::Run => "Pass a source or project path after `fpas run`, or `fpas --help`.",
                CliMode::Debug if arg == "--report" && options.debug_protocol.is_some() => {
                    "JSONL is already the protocol output; use `fpas debug <target> --protocol jsonl --commands <path>`."
                }
                CliMode::Debug if arg == "--report" => {
                    "Batch already returns JSON; use `fpas debug <target>` or select an explicit debugger protocol."
                }
                CliMode::Debug => {
                    "Pass a debug target after `fpas debug`, or use `fpas debug --help`."
                }
                CliMode::Env => {
                    unreachable!("environment command handled before shared option parsing")
                }
                CliMode::Check => {
                    "Pass a source or project path after `fpas check`, or `fpas --help`."
                }
                CliMode::Test => "Pass a test path after `fpas test`, or `fpas --help`.",
                CliMode::Fmt => unreachable!("fmt mode handled above"),
            };
            return Err(format!("Unknown option `{arg}`.\n  help: {hint}"));
        }

        if input.replace(arg.clone()).is_some() {
            return Err(usage_error(mode));
        }
    }

    let input = match input {
        Some(input) => resolve_explicit_input(&input, cwd, mode),
        None => discover_input(cwd, mode),
    }?;

    if mode == CliMode::Build && options.application_name.is_some() && !options.executable {
        return Err(
            "`--name` requires `--executable`.\n  help: `fpas build --executable --name hello hello.fpasprj`."
                .to_string(),
        );
    }

    Ok(match mode {
        CliMode::Init | CliMode::Doc => {
            unreachable!("init and doc are handled before shared option parsing")
        }
        CliMode::Build => ResolvedCli::Build(BuildCliConfig {
            no_contracts: options.no_contracts,
            input,
            standard_library: options.standard_library,
            executable: options.executable,
            name: options.application_name,
            diagnostics,
        }),
        CliMode::Run => ResolvedCli::Run(CliConfig {
            no_contracts: options.no_contracts,
            collection_schedule: options.collection_schedule,
            input,
            program_args,
            standard_library: options.standard_library,
            diagnostics,
            failure_dump,
            profile,
            fix: false,
        }),
        CliMode::Debug => {
            let protocol = options.debug_protocol.unwrap_or(DebugProtocol::Batch);
            let mut batch = options.batch;
            batch.script = options.script_path;
            if batch.trace_limit.is_some() && batch.trace.is_empty() {
                return Err("--trace-limit requires at least one --trace.".into());
            }
            if protocol != DebugProtocol::Batch && batch != Default::default() {
                return Err(
                    "Batch inspection/input options cannot be combined with --protocol.".into(),
                );
            }
            if protocol != DebugProtocol::Jsonl && options.commands.is_some() {
                return Err("--commands requires --protocol jsonl.".into());
            }
            if protocol == DebugProtocol::Batch
                && options.output_limit.is_some_and(|limit| limit > 1_048_576)
            {
                return Err("Batch --output-limit cannot exceed 1048576 bytes.".into());
            }
            ResolvedCli::Debug(Box::new(DebugCliConfig {
                no_contracts: options.no_contracts,
                profile,
                collection_schedule: options.collection_schedule,
                batch,
                cwd: cwd.to_path_buf(),
                input,
                program_args,
                standard_library: options.standard_library,
                protocol,
                commands: options.commands,
                source_root: options.source_root,
                timeout: options.timeout.unwrap_or(Duration::from_secs(
                    if protocol == DebugProtocol::Batch {
                        30
                    } else {
                        300
                    },
                )),
                instruction_limit: options.instruction_limit.unwrap_or(100_000_000),
                output_limit: options.output_limit.unwrap_or(1_048_576),
            }))
        }
        CliMode::Check => ResolvedCli::Check(CliConfig {
            no_contracts: false,
            collection_schedule: options.collection_schedule,
            input,
            program_args,
            standard_library: options.standard_library,
            diagnostics,
            failure_dump: None,
            profile: None,
            fix: options.fix,
        }),
        CliMode::Fmt => unreachable!("fmt mode handled above"),
        CliMode::Test => ResolvedCli::Test(TestCliConfig {
            no_contracts: options.no_contracts,
            collection_schedule: options.collection_schedule,
            input,
            cwd: cwd.to_path_buf(),
            fail_fast: options.fail_fast,
            list_only: options.list_only,
            script_path: options.script_path,
            filter: options.filter,
            files: options.files,
            report: options.report,
            timeout: Some(options.timeout.unwrap_or(DEFAULT_TEST_TIMEOUT)),
            jobs: options.jobs.unwrap_or(1),
            strict: options.strict,
            show_output: options.show_output,
            diagnostics,
            standard_library: options.standard_library,
        }),
        CliMode::Env => {
            unreachable!("environment command handled before input discovery")
        }
    })
}

fn resolve_environment_command(args: &[String]) -> Result<ResolvedCli, String> {
    if matches!(args, [argument] if argument == "-h" || argument == "--help") {
        return Ok(ResolvedCli::Help(HelpTopic::Env));
    }
    if matches!(args, [argument] if argument == "--json") {
        Ok(ResolvedCli::Environment)
    } else {
        Err(usage_error(CliMode::Env))
    }
}
