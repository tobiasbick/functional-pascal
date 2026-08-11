//! Argument parsing for the nested `fpas init` command.

use std::path::{Path, PathBuf};

use crate::cli_init::naming::{is_available_identifier, pascal_identifier};

use super::{HelpTopic, InitCliConfig, InitKind, InitReportFormat, ResolvedCli};

/// Resolves an `init` scaffold kind, name, and non-interactive options.
pub(super) fn resolve_init_cli(args: &[String], cwd: &Path) -> Result<ResolvedCli, String> {
    let Some(kind_arg) = args.first() else {
        return Ok(ResolvedCli::Help(HelpTopic::Init));
    };

    if matches!(kind_arg.as_str(), "-h" | "--help") {
        if args.len() == 1 {
            return Ok(ResolvedCli::Help(HelpTopic::Init));
        }
        return Err(init_usage_error());
    }
    if matches!(kind_arg.as_str(), "-V" | "--version") {
        if args.len() == 1 {
            return Ok(ResolvedCli::Version);
        }
        return Err(init_usage_error());
    }

    let kind = parse_kind(kind_arg)?;
    let kind_help = help_topic(kind);
    if args.len() == 2 && matches!(args[1].as_str(), "-h" | "--help") {
        return Ok(ResolvedCli::Help(kind_help));
    }

    let mut name = None::<String>;
    let mut path = None::<PathBuf>;
    let mut unit = None::<String>;
    let mut dry_run = false;
    let mut report = None::<InitReportFormat>;
    let mut index = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--path" => {
                let value = take_value(args, &mut index, "--path", kind)?;
                if value.is_empty() {
                    return Err(format!(
                        "`--path` cannot be empty.\n  help: `{}`.",
                        example(kind)
                    ));
                }
                if path.replace(PathBuf::from(value)).is_some() {
                    return Err("Duplicate `--path` option.".to_string());
                }
            }
            "--unit" if kind == InitKind::Library => {
                let value = take_value(args, &mut index, "--unit", kind)?;
                validate_unit_name(value)?;
                if unit.replace(value.to_string()).is_some() {
                    return Err("Duplicate `--unit` option.".to_string());
                }
            }
            "--dry-run" => {
                if dry_run {
                    return Err("Duplicate `--dry-run` option.".to_string());
                }
                dry_run = true;
            }
            "--report" => {
                let value = take_value(args, &mut index, "--report", kind)?;
                if value != "json" {
                    return Err(format!(
                        "Unsupported init report format `{value}`.\n  help: Use `--report json`."
                    ));
                }
                if report.replace(InitReportFormat::Json).is_some() {
                    return Err("Duplicate `--report` option.".to_string());
                }
            }
            "-h" | "--help" => {
                return Err(format!(
                    "`--help` cannot be combined with scaffold arguments.\n  help: `fpas init {} --help`.",
                    kind.as_str()
                ));
            }
            "-V" | "--version" => {
                return Err(
                    "`--version` cannot be combined with scaffold arguments.\n  help: Use `fpas --version`."
                        .to_string(),
                );
            }
            arg if arg.starts_with('-') => {
                return Err(format!(
                    "Unknown option `{arg}` for `fpas init {}`.\n  help: `fpas init {} --help` lists valid options.",
                    kind.as_str(),
                    kind.as_str()
                ));
            }
            value => {
                if name.replace(value.to_string()).is_some() {
                    return Err(format!(
                        "Too many names for `fpas init {}`.\n  help: `{}`.",
                        kind.as_str(),
                        example(kind)
                    ));
                }
            }
        }
        index += 1;
    }

    let name = name.ok_or_else(|| {
        format!(
            "Missing name for `fpas init {}`.\n  help: `{}`.",
            kind.as_str(),
            example(kind)
        )
    })?;
    validate_project_name(&name)?;
    let identifier = pascal_identifier(&name);
    if !is_available_identifier(&identifier) {
        return Err(format!(
            "Scaffold name `{name}` derives the reserved Functional Pascal identifier `{identifier}`.\n  help: Choose a more specific name, for example `{name}-app`."
        ));
    }

    let requested_root = path.unwrap_or_else(|| PathBuf::from(&name));
    let root = if requested_root.is_absolute() {
        requested_root
    } else {
        cwd.join(requested_root)
    };

    Ok(ResolvedCli::Init(InitCliConfig {
        cwd: cwd.to_path_buf(),
        kind,
        name,
        root,
        library_unit: unit,
        dry_run,
        report,
    }))
}

fn parse_kind(value: &str) -> Result<InitKind, String> {
    match value {
        "project" => Ok(InitKind::Project),
        "library" => Ok(InitKind::Library),
        "workspace" => Ok(InitKind::Workspace),
        _ => Err(format!(
            "Unknown scaffold kind `{value}`. Expected `project`, `library`, or `workspace`.\n  help: `fpas init --help` lists examples."
        )),
    }
}

fn help_topic(kind: InitKind) -> HelpTopic {
    match kind {
        InitKind::Project => HelpTopic::InitProject,
        InitKind::Library => HelpTopic::InitLibrary,
        InitKind::Workspace => HelpTopic::InitWorkspace,
    }
}

fn take_value<'a>(
    args: &'a [String],
    index: &mut usize,
    option: &str,
    kind: InitKind,
) -> Result<&'a str, String> {
    *index += 1;
    let value = args.get(*index).ok_or_else(|| {
        format!(
            "Missing value after `{option}`.\n  help: `{}`.",
            example(kind)
        )
    })?;
    if is_known_option(value) {
        return Err(format!(
            "Missing value after `{option}`: `{value}` is an option and cannot be the value.\n  help: `{}`.",
            example(kind)
        ));
    }
    Ok(value)
}

fn is_known_option(value: &str) -> bool {
    matches!(
        value,
        "--path" | "--unit" | "--dry-run" | "--report" | "-h" | "--help" | "-V" | "--version"
    )
}

fn validate_project_name(name: &str) -> Result<(), String> {
    let mut previous_was_separator = false;
    for (index, byte) in name.bytes().enumerate() {
        let is_separator = matches!(byte, b'-' | b'_');
        let valid = if index == 0 {
            byte.is_ascii_alphabetic()
        } else {
            byte.is_ascii_alphanumeric() || is_separator
        };
        if !valid || (is_separator && previous_was_separator) {
            return Err(invalid_project_name_error(name));
        }
        previous_was_separator = is_separator;
    }
    if name.is_empty() || previous_was_separator {
        return Err(invalid_project_name_error(name));
    }
    Ok(())
}

fn invalid_project_name_error(name: &str) -> String {
    format!(
        "Invalid scaffold name `{name}`. Use ASCII letters and digits with single `-` or `_` separators, starting with a letter.\n  help: For example, `fpas init project my-app`."
    )
}

fn validate_unit_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.split('.').any(|part| !is_available_identifier(part)) {
        return Err(format!(
            "Invalid Functional Pascal unit name `{name}`.\n  help: Use dot-separated ASCII identifiers, for example `--unit Acme.Greet`."
        ));
    }
    Ok(())
}

fn example(kind: InitKind) -> &'static str {
    match kind {
        InitKind::Project => "fpas init project hello",
        InitKind::Library => "fpas init library greet --unit Demo.Greet",
        InitKind::Workspace => "fpas init workspace acme-suite",
    }
}

fn init_usage_error() -> String {
    "Usage: fpas init <project | library | workspace> <name> [options]\n  help: `fpas init --help` shows scaffold kinds and examples."
        .to_string()
}
