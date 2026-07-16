//! CLI argument resolution and project discovery.
//!
//! Spec: [Projects & CLI](../../../docs/pascal/program-structure/cli.md).

mod discovery;
mod help;
mod mode;
mod types;

use std::path::Path;
use std::time::Duration;

pub(crate) use discovery::discover_check_input;
pub(crate) use help::CLI_HELP;
pub(crate) use types::{
    CliConfig, CliInput, FmtCliConfig, ResolvedCli, TestCliConfig, TestReportFormat,
};

use mode::{
    CliMode, parse_cli_mode, program_args_require_run_error, split_program_args, usage_error,
};

use discovery::{discover_input, resolve_explicit_input};

/// Resolves at most one positional path or project discovery, for unit tests only.
///
/// [`resolve_cli_config`] is the full CLI entry: it also handles `--help` / `--version`.
#[cfg(test)]
pub(crate) fn resolve_cli_input(args: &[String], cwd: &Path) -> Result<CliInput, String> {
    match resolve_cli_config(args, cwd)? {
        ResolvedCli::Run(config) | ResolvedCli::Check(config) => Ok(config.input),
        ResolvedCli::Fmt(_) => {
            Err("resolve_cli_input: use resolve_cli_config for `fpas fmt`".to_string())
        }
        ResolvedCli::Test(_) | ResolvedCli::Help | ResolvedCli::Version => {
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
        return Ok(ResolvedCli::Help);
    }

    if cli_args.len() == 1 {
        match cli_args[0].as_str() {
            "-h" | "--help" => return Ok(ResolvedCli::Help),
            "-V" | "--version" => return Ok(ResolvedCli::Version),
            _ => {}
        }
    }

    let (mode, cli_args) = parse_cli_mode(cli_args)?;

    if matches!(mode, CliMode::Check | CliMode::Fmt | CliMode::Test) && !program_args.is_empty() {
        let cmd = match mode {
            CliMode::Check => "fpas check",
            CliMode::Fmt => "fpas fmt",
            CliMode::Test => "fpas test",
            CliMode::Run => unreachable!(),
        };
        return Err(format!(
            "`{cmd}` does not accept program arguments after `--`.\n  help: Omit `--` and trailing args when type-checking or testing."
        ));
    }

    let mut check_only = false;
    let mut stdout_mode = false;
    let mut list_changed = false;
    let mut fail_fast = false;
    let mut strict = false;
    let mut list_only = false;
    let mut script_path = None::<std::path::PathBuf>;
    let mut filter = None::<String>;
    let mut report = None::<TestReportFormat>;
    let mut timeout = None::<Duration>;
    let mut jobs = None::<usize>;
    let mut standard_library = None::<std::path::PathBuf>;
    let mut positional = Vec::new();
    let mut index = 0;
    while index < cli_args.len() {
        match cli_args[index].as_str() {
            "--std-lib" if matches!(mode, CliMode::Run | CliMode::Check | CliMode::Test) => {
                index += 1;
                let Some(path) = cli_args.get(index) else {
                    return Err(
                        "Missing directory after `--std-lib`.\n  help: `fpas run --std-lib ./lib hello.fpas`."
                            .to_string(),
                    );
                };
                if standard_library
                    .replace(std::path::PathBuf::from(path))
                    .is_some()
                {
                    return Err("Duplicate `--std-lib` option.".to_string());
                }
            }
            "--check" if mode == CliMode::Fmt => check_only = true,
            "--stdout" if mode == CliMode::Fmt => {
                if stdout_mode {
                    return Err("Duplicate `--stdout` option.".to_string());
                }
                stdout_mode = true;
            }
            "--list" if mode == CliMode::Fmt => {
                if list_changed {
                    return Err("Duplicate `--list` option.".to_string());
                }
                list_changed = true;
            }
            "--fail-fast" if mode == CliMode::Test => fail_fast = true,
            "--strict" if mode == CliMode::Test => strict = true,
            "--list" if mode == CliMode::Test => list_only = true,
            "--script" if mode == CliMode::Test => {
                index += 1;
                let Some(path) = cli_args.get(index) else {
                    return Err(
                        "Missing path after `--script`.\n  help: `fpas test --script menu.script.toml`."
                            .to_string(),
                    );
                };
                if script_path
                    .replace(std::path::PathBuf::from(path))
                    .is_some()
                {
                    return Err("Duplicate `--script` option.".to_string());
                }
            }
            "--filter" if mode == CliMode::Test => {
                index += 1;
                let Some(pattern) = cli_args.get(index) else {
                    return Err(
                        "Missing pattern after `--filter`.\n  help: `fpas test --filter menu`."
                            .to_string(),
                    );
                };
                if filter.replace(pattern.clone()).is_some() {
                    return Err("Duplicate `--filter` option.".to_string());
                }
            }
            "--report" if mode == CliMode::Test => {
                index += 1;
                let Some(format) = cli_args.get(index) else {
                    return Err(
                        "Missing format after `--report`.\n  help: `fpas test --report json`."
                            .to_string(),
                    );
                };
                if format != "json" {
                    return Err(format!(
                        "Unsupported report format `{format}`.\n  help: Only `--report json` is supported."
                    ));
                }
                if report.replace(TestReportFormat::Json).is_some() {
                    return Err("Duplicate `--report` option.".to_string());
                }
            }
            "--timeout" if mode == CliMode::Test => {
                index += 1;
                let Some(secs) = cli_args.get(index) else {
                    return Err(
                        "Missing seconds after `--timeout`.\n  help: `fpas test --timeout 30`."
                            .to_string(),
                    );
                };
                let secs: u64 = secs.parse().map_err(|_| {
                    format!(
                        "Invalid `--timeout` value `{secs}`.\n  help: Pass a positive integer number of seconds."
                    )
                })?;
                if secs == 0 {
                    return Err(
                        "`--timeout` must be at least 1 second.\n  help: `fpas test --timeout 30`."
                            .to_string(),
                    );
                }
                if timeout.replace(Duration::from_secs(secs)).is_some() {
                    return Err("Duplicate `--timeout` option.".to_string());
                }
            }
            "--jobs" if mode == CliMode::Test => {
                index += 1;
                let Some(count) = cli_args.get(index) else {
                    return Err(
                        "Missing count after `--jobs`.\n  help: `fpas test --jobs 4` or `fpas test --jobs 0` for machine parallelism."
                            .to_string(),
                    );
                };
                let count: usize = count.parse().map_err(|_| {
                    format!(
                        "Invalid `--jobs` value `{count}`.\n  help: Pass a non-negative integer (`0` uses available parallelism)."
                    )
                })?;
                if jobs.replace(count).is_some() {
                    return Err("Duplicate `--jobs` option.".to_string());
                }
            }
            _ => positional.push(cli_args[index].clone()),
        }
        index += 1;
    }

    if mode == CliMode::Fmt {
        if stdout_mode && check_only {
            return Err(
                "`fpas fmt --stdout` cannot be combined with `--check`.\n  help: Use `--stdout` to print formatted output, or `--check` to verify on disk."
                    .to_string(),
            );
        }
        if list_changed && !check_only {
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
                return Ok(ResolvedCli::Help);
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
            check_only,
            stdout: stdout_mode,
            list_changed,
        }));
    }

    let mut input = None::<String>;
    for arg in &positional {
        if arg == "-h" || arg == "--help" {
            if positional.len() != 1 {
                return Err(usage_error(mode));
            }
            return Ok(ResolvedCli::Help);
        }
        if arg == "-V" || arg == "--version" {
            if positional.len() != 1 {
                return Err(usage_error(mode));
            }
            return Ok(ResolvedCli::Version);
        }

        if arg.starts_with('-') {
            let hint = match mode {
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

    Ok(match mode {
        CliMode::Run => ResolvedCli::Run(CliConfig {
            input,
            program_args,
            standard_library,
        }),
        CliMode::Check => ResolvedCli::Check(CliConfig {
            input,
            program_args,
            standard_library,
        }),
        CliMode::Fmt => unreachable!("fmt mode handled above"),
        CliMode::Test => ResolvedCli::Test(TestCliConfig {
            input,
            cwd: cwd.to_path_buf(),
            fail_fast,
            list_only,
            script_path,
            filter,
            report,
            timeout,
            jobs: jobs.unwrap_or(1),
            strict,
            standard_library,
        }),
    })
}
