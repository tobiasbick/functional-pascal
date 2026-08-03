//! CLI argument resolution and project discovery.
//!
//! Spec: [Projects & CLI](../../../docs/pascal/program-structure/cli.md).

mod discovery;
mod help;
mod mode;
mod options;
mod types;

use std::path::Path;

pub(crate) use discovery::discover_check_input;
pub(crate) use help::help_text;
pub(crate) use types::{
    BuildCliConfig, CliConfig, CliInput, FmtCliConfig, HelpTopic, ResolvedCli, TestCliConfig,
    TestReportFormat,
};

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
        ResolvedCli::Build(config) => Ok(config.input),
        ResolvedCli::Run(config) | ResolvedCli::Check(config) => Ok(config.input),
        ResolvedCli::Fmt(_) => {
            Err("resolve_cli_input: use resolve_cli_config for `fpas fmt`".to_string())
        }
        ResolvedCli::Test(_) | ResolvedCli::Help(_) | ResolvedCli::Version => {
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

    let (mode, cli_args) = parse_cli_mode(cli_args)?;

    if mode != CliMode::Run && !program_args.is_empty() {
        let cmd = match mode {
            CliMode::Build => "fpas build",
            CliMode::Check => "fpas check",
            CliMode::Fmt => "fpas fmt",
            CliMode::Test => "fpas test",
            CliMode::Run => unreachable!(),
        };
        return Err(format!(
            "`{cmd}` does not accept program arguments after `--`.\n  help: Omit `--` and trailing program arguments."
        ));
    }

    let options = parse_options(mode, cli_args)?;
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
                CliMode::Build => {
                    "Pass a project or workspace path after `fpas build`, or use `fpas build --help`."
                }
                CliMode::Run => "Pass a source or project path after `fpas run`, or `fpas --help`.",
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
        CliMode::Build => ResolvedCli::Build(BuildCliConfig {
            input,
            standard_library: options.standard_library,
            executable: options.executable,
            name: options.application_name,
        }),
        CliMode::Run => ResolvedCli::Run(CliConfig {
            input,
            program_args,
            standard_library: options.standard_library,
        }),
        CliMode::Check => ResolvedCli::Check(CliConfig {
            input,
            program_args,
            standard_library: options.standard_library,
        }),
        CliMode::Fmt => unreachable!("fmt mode handled above"),
        CliMode::Test => ResolvedCli::Test(TestCliConfig {
            input,
            cwd: cwd.to_path_buf(),
            fail_fast: options.fail_fast,
            list_only: options.list_only,
            script_path: options.script_path,
            filter: options.filter,
            report: options.report,
            timeout: options.timeout,
            jobs: options.jobs.unwrap_or(1),
            strict: options.strict,
            standard_library: options.standard_library,
        }),
    })
}
