//! Subcommand option parsing and value validation.

use std::path::PathBuf;
use std::time::Duration;

use super::mode::CliMode;
use super::types::{DebugProtocol, TestReportFormat};

pub(super) struct ParsedOptions {
    pub(super) check_only: bool,
    pub(super) stdout_mode: bool,
    pub(super) list_changed: bool,
    pub(super) fail_fast: bool,
    pub(super) strict: bool,
    pub(super) list_only: bool,
    pub(super) script_path: Option<PathBuf>,
    pub(super) filter: Option<String>,
    pub(super) report: Option<TestReportFormat>,
    pub(super) timeout: Option<Duration>,
    pub(super) jobs: Option<usize>,
    pub(super) standard_library: Option<PathBuf>,
    pub(super) executable: bool,
    pub(super) application_name: Option<String>,
    pub(super) debug_protocol: Option<DebugProtocol>,
    pub(super) commands: Option<PathBuf>,
    pub(super) source_root: Option<PathBuf>,
    pub(super) instruction_limit: Option<u64>,
    pub(super) output_limit: Option<usize>,
    pub(super) positional: Vec<String>,
}

/// Parses the options accepted by one CLI subcommand.
pub(super) fn parse_options(mode: CliMode, cli_args: &[String]) -> Result<ParsedOptions, String> {
    let mut options = ParsedOptions {
        check_only: false,
        stdout_mode: false,
        list_changed: false,
        fail_fast: false,
        strict: false,
        list_only: false,
        script_path: None,
        filter: None,
        report: None,
        timeout: None,
        jobs: None,
        standard_library: None,
        executable: false,
        application_name: None,
        debug_protocol: None,
        commands: None,
        source_root: None,
        instruction_limit: None,
        output_limit: None,
        positional: Vec::new(),
    };

    let mut index = 0;
    while index < cli_args.len() {
        match cli_args[index].as_str() {
            "--std-lib"
                if matches!(
                    mode,
                    CliMode::Build | CliMode::Run | CliMode::Debug | CliMode::Check | CliMode::Test
                ) =>
            {
                let example = match mode {
                    CliMode::Build => "fpas build --std-lib ./lib my-app.fpasprj",
                    CliMode::Run => "fpas run --std-lib ./lib hello.fpas",
                    CliMode::Debug => "fpas debug --std-lib ./lib hello.fpas --protocol jsonl",
                    CliMode::Check => "fpas check --std-lib ./lib my-app.fpasprj",
                    CliMode::Test => "fpas test --std-lib ./lib tests/",
                    CliMode::Init => unreachable!("init has its own option parser"),
                    CliMode::Fmt => unreachable!("fmt does not accept --std-lib"),
                };
                let path = take_option_value(
                    cli_args,
                    &mut index,
                    "--std-lib",
                    &format!("Missing directory after `--std-lib`.\n  help: `{example}`."),
                )?;
                if options
                    .standard_library
                    .replace(PathBuf::from(path))
                    .is_some()
                {
                    return Err("Duplicate `--std-lib` option.".to_string());
                }
            }
            "--executable" if mode == CliMode::Build => {
                if options.executable {
                    return Err("Duplicate `--executable` option.".to_string());
                }
                options.executable = true;
            }
            "--name" if mode == CliMode::Build => {
                let name = take_option_value(
                    cli_args,
                    &mut index,
                    "--name",
                    "Missing application name after `--name`.\n  help: `fpas build --executable --name hello hello.fpasprj`.",
                )?;
                if options.application_name.replace(name.to_string()).is_some() {
                    return Err("Duplicate `--name` option.".to_string());
                }
            }
            "--check" if mode == CliMode::Fmt => options.check_only = true,
            "--stdout" if mode == CliMode::Fmt => {
                if options.stdout_mode {
                    return Err("Duplicate `--stdout` option.".to_string());
                }
                options.stdout_mode = true;
            }
            "--list" if mode == CliMode::Fmt => {
                if options.list_changed {
                    return Err("Duplicate `--list` option.".to_string());
                }
                options.list_changed = true;
            }
            "--fail-fast" if mode == CliMode::Test => options.fail_fast = true,
            "--strict" if mode == CliMode::Test => options.strict = true,
            "--list" if mode == CliMode::Test => options.list_only = true,
            "--script" if mode == CliMode::Test => {
                let path = take_option_value(
                    cli_args,
                    &mut index,
                    "--script",
                    "Missing path after `--script`.\n  help: `fpas test --script menu.script.toml`.",
                )?;
                if options.script_path.replace(PathBuf::from(path)).is_some() {
                    return Err("Duplicate `--script` option.".to_string());
                }
            }
            "--filter" if mode == CliMode::Test => {
                let pattern = take_option_value(
                    cli_args,
                    &mut index,
                    "--filter",
                    "Missing pattern after `--filter`.\n  help: `fpas test --filter menu`.",
                )?;
                if options.filter.replace(pattern.to_string()).is_some() {
                    return Err("Duplicate `--filter` option.".to_string());
                }
            }
            "--report" if mode == CliMode::Test => {
                let format = take_option_value(
                    cli_args,
                    &mut index,
                    "--report",
                    "Missing format after `--report`.\n  help: `fpas test --report json`.",
                )?;
                if format != "json" {
                    return Err(format!(
                        "Unsupported report format `{format}`.\n  help: Only `--report json` is supported."
                    ));
                }
                if options.report.replace(TestReportFormat::Json).is_some() {
                    return Err("Duplicate `--report` option.".to_string());
                }
            }
            "--timeout" if mode == CliMode::Test => {
                let secs = take_option_value(
                    cli_args,
                    &mut index,
                    "--timeout",
                    "Missing seconds after `--timeout`.\n  help: `fpas test --timeout 30`.",
                )?;
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
                if options.timeout.replace(Duration::from_secs(secs)).is_some() {
                    return Err("Duplicate `--timeout` option.".to_string());
                }
            }
            "--protocol" if mode == CliMode::Debug => {
                let value = take_option_value(
                    cli_args,
                    &mut index,
                    "--protocol",
                    "Missing protocol after `--protocol`.\n  help: Use `--protocol jsonl` or `--protocol dap`.",
                )?;
                let protocol = match value {
                    "jsonl" => DebugProtocol::Jsonl,
                    "dap" => DebugProtocol::Dap,
                    _ => {
                        return Err(format!(
                            "Unsupported debugger protocol `{value}`.\n  help: Use `--protocol jsonl` or `--protocol dap`."
                        ));
                    }
                };
                if options.debug_protocol.replace(protocol).is_some() {
                    return Err("Duplicate `--protocol` option.".to_string());
                }
            }
            "--commands" if mode == CliMode::Debug => {
                let value = take_option_value(
                    cli_args,
                    &mut index,
                    "--commands",
                    "Missing path after `--commands`.\n  help: `fpas debug hello.fpas --protocol jsonl --commands session.jsonl`.",
                )?;
                if options.commands.replace(PathBuf::from(value)).is_some() {
                    return Err("Duplicate `--commands` option.".to_string());
                }
            }
            "--source-root" if mode == CliMode::Debug => {
                let value = take_option_value(
                    cli_args,
                    &mut index,
                    "--source-root",
                    "Missing directory after `--source-root`.\n  help: Compiled images require their verified source root.",
                )?;
                if options.source_root.replace(PathBuf::from(value)).is_some() {
                    return Err("Duplicate `--source-root` option.".to_string());
                }
            }
            "--timeout" if mode == CliMode::Debug => {
                let value = positive_u64(cli_args, &mut index, "--timeout", "seconds")?;
                options.timeout = Some(Duration::from_secs(value));
            }
            "--instruction-limit" if mode == CliMode::Debug => {
                options.instruction_limit = Some(positive_u64(
                    cli_args,
                    &mut index,
                    "--instruction-limit",
                    "instructions",
                )?);
            }
            "--output-limit" if mode == CliMode::Debug => {
                let value = positive_u64(cli_args, &mut index, "--output-limit", "bytes")?;
                options.output_limit = Some(
                    usize::try_from(value)
                        .map_err(|_| "`--output-limit` is too large for this host.".to_string())?,
                );
            }
            "--report" if mode == CliMode::Debug => {
                let value = take_option_value(
                    cli_args,
                    &mut index,
                    "--report",
                    "Missing format after `--report`.\n  help: Script mode supports `--report jsonl`.",
                )?;
                if value != "jsonl" {
                    return Err(format!(
                        "Unsupported debugger report `{value}`.\n  help: Use `--report jsonl`."
                    ));
                }
            }
            "--jobs" if mode == CliMode::Test => {
                let count = take_option_value(
                    cli_args,
                    &mut index,
                    "--jobs",
                    "Missing count after `--jobs`.\n  help: `fpas test --jobs 4` or `fpas test --jobs 0` for machine parallelism.",
                )?;
                let count: usize = count.parse().map_err(|_| {
                    format!(
                        "Invalid `--jobs` value `{count}`.\n  help: Pass a non-negative integer (`0` uses available parallelism)."
                    )
                })?;
                if options.jobs.replace(count).is_some() {
                    return Err("Duplicate `--jobs` option.".to_string());
                }
            }
            _ => options.positional.push(cli_args[index].clone()),
        }
        index += 1;
    }

    Ok(options)
}

fn take_option_value<'a>(
    args: &'a [String],
    index: &mut usize,
    option: &str,
    missing_message: &str,
) -> Result<&'a str, String> {
    *index += 1;
    let Some(value) = args.get(*index) else {
        return Err(missing_message.to_string());
    };
    if is_known_option(value) {
        return Err(format!(
            "{missing_message}\n  note: `{value}` is an option and cannot be the value for `{option}`."
        ));
    }
    Ok(value)
}

fn is_known_option(value: &str) -> bool {
    matches!(
        value,
        "--std-lib"
            | "--executable"
            | "--name"
            | "--check"
            | "--stdout"
            | "--list"
            | "--fail-fast"
            | "--strict"
            | "--script"
            | "--filter"
            | "--report"
            | "--timeout"
            | "--jobs"
            | "--protocol"
            | "--commands"
            | "--source-root"
            | "--instruction-limit"
            | "--output-limit"
            | "-h"
            | "--help"
            | "-V"
            | "--version"
    )
}

fn positive_u64(
    args: &[String],
    index: &mut usize,
    option: &str,
    unit: &str,
) -> Result<u64, String> {
    let value = take_option_value(
        args,
        index,
        option,
        &format!("Missing value after `{option}`."),
    )?;
    value.parse::<u64>().ok().filter(|value| *value > 0).ok_or_else(|| format!("Invalid `{option}` value `{value}`.\n  help: Pass a positive integer number of {unit}."))
}
