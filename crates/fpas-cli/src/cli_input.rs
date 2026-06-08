//! CLI argument resolution and project discovery.
//!
//! Spec: [Projects & CLI](../../../docs/pascal/10-projects.md).

use fpas_project::{discover_run_project_in_workspace, discover_workspace_file};
use std::fs;
use std::path::{Path, PathBuf};

const SOURCE_FILE_EXTENSION: &str = "fpas";
const PROJECT_FILE_EXTENSION: &str = "fpasprj";
const WORKSPACE_FILE_EXTENSION: &str = "fpasworkspace";

/// Text printed for `fpas -h` / `fpas --help` (stdout).
pub(crate) const CLI_HELP: &str = "\
fpas — Functional Pascal compiler

Usage:
    fpas [<file.fpas | file.fpasprj>] [-- <args>...]       Run a source file or project
    fpas [-- <args>...]                                   Discover a workspace program or `.fpasprj` in cwd
    fpas check [<file.fpas | file.fpasprj | file.fpasworkspace>]
                                                          Type-check without running
    fpas check                                            Discover `.fpasworkspace` or `.fpasprj` in cwd
    fpas test [<file.fpas | dir | file.fpasprj | file.fpasworkspace>]
                                                          Run `*_test.fpas` programs
    fpas test [--list] [--fail-fast] [--script <path>] [<path>]             Discover tests in cwd when path omitted

Options:
  -h, --help      Print this help
  -V, --version   Print version

Program arguments after `--` are visible through `Std.Args` when running programs.

";

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum CliInput {
    SourceFile(PathBuf),
    ProjectFile(PathBuf),
    WorkspaceFile(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CliConfig {
    pub input: CliInput,
    pub program_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TestCliConfig {
    pub input: CliInput,
    pub cwd: PathBuf,
    pub fail_fast: bool,
    pub list_only: bool,
    pub script_path: Option<PathBuf>,
}

/// Result of parsing CLI arguments before loading sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedCli {
    Run(CliConfig),
    Check(CliConfig),
    Test(TestCliConfig),
    Help,
    Version,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CliMode {
    Run,
    Check,
    Test,
}

/// Resolves at most one positional path or project discovery, for unit tests only.
///
/// [`resolve_cli_config`] is the full CLI entry: it also handles `--help` / `--version`.
#[cfg(test)]
pub(crate) fn resolve_cli_input(args: &[String], cwd: &Path) -> Result<CliInput, String> {
    match resolve_cli_config(args, cwd)? {
        ResolvedCli::Run(config) | ResolvedCli::Check(config) => Ok(config.input),
        ResolvedCli::Test(_) | ResolvedCli::Help | ResolvedCli::Version => {
            Err("resolve_cli_input: use resolve_cli_config for --help or --version".to_string())
        }
    }
}

pub(crate) fn resolve_cli_config(args: &[String], cwd: &Path) -> Result<ResolvedCli, String> {
    let (cli_args, program_args) = split_program_args(args);
    let (mode, cli_args) = parse_cli_mode(cli_args)?;

    if matches!(mode, CliMode::Check | CliMode::Test) && !program_args.is_empty() {
        let cmd = match mode {
            CliMode::Check => "fpas check",
            CliMode::Test => "fpas test",
            CliMode::Run => unreachable!(),
        };
        return Err(format!(
            "`{cmd}` does not accept program arguments after `--`.\n  help: Omit `--` and trailing args when type-checking or testing."
        ));
    }

    let mut fail_fast = false;
    let mut list_only = false;
    let mut script_path = None::<PathBuf>;
    let mut positional = Vec::new();
    let mut index = 0;
    while index < cli_args.len() {
        match cli_args[index].as_str() {
            "--fail-fast" if mode == CliMode::Test => fail_fast = true,
            "--list" if mode == CliMode::Test => list_only = true,
            "--script" if mode == CliMode::Test => {
                index += 1;
                let Some(path) = cli_args.get(index) else {
                    return Err(
                        "Missing path after `--script`.\n  help: `fpas test --script menu.script.toml`."
                            .to_string(),
                    );
                };
                if script_path.replace(PathBuf::from(path)).is_some() {
                    return Err("Duplicate `--script` option.".to_string());
                }
            }
            _ => positional.push(cli_args[index].clone()),
        }
        index += 1;
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
            return Err(format!(
                "Unknown option `{arg}`.\n  help: Pass a source or project path, or `fpas --help`."
            ));
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
        }),
        CliMode::Check => ResolvedCli::Check(CliConfig {
            input,
            program_args,
        }),
        CliMode::Test => ResolvedCli::Test(TestCliConfig {
            input,
            cwd: cwd.to_path_buf(),
            fail_fast,
            list_only,
            script_path,
        }),
    })
}

fn parse_cli_mode(cli_args: &[String]) -> Result<(CliMode, &[String]), String> {
    if cli_args.first().is_some_and(|arg| arg == "check") {
        return Ok((CliMode::Check, &cli_args[1..]));
    }
    if cli_args.first().is_some_and(|arg| arg == "test") {
        return Ok((CliMode::Test, &cli_args[1..]));
    }

    Ok((CliMode::Run, cli_args))
}

fn usage_error(mode: CliMode) -> String {
    match mode {
        CliMode::Run => {
            "Usage: fpas [<file.fpas | file.fpasprj>] [-- <args>...]\n  help: `fpas --help` shows options."
                .to_string()
        }
        CliMode::Check => {
            "Usage: fpas check [<file.fpas | file.fpasprj | file.fpasworkspace>]\n  help: `fpas --help` shows options."
                .to_string()
        }
        CliMode::Test => {
            "Usage: fpas test [--list] [--fail-fast] [--script <path>] [<file.fpas | dir | file.fpasprj | file.fpasworkspace>]\n  help: `fpas --help` shows options."
                .to_string()
        }
    }
}

fn split_program_args(args: &[String]) -> (&[String], Vec<String>) {
    let Some(separator) = args.iter().position(|arg| arg == "--") else {
        return (args, Vec::new());
    };

    (&args[..separator], args[separator + 1..].to_vec())
}

fn resolve_explicit_input(input: &str, cwd: &Path, mode: CliMode) -> Result<CliInput, String> {
    let path = normalize_input_path(input, cwd);
    if path.is_dir() {
        return Ok(CliInput::SourceFile(path));
    }
    if has_extension(&path, SOURCE_FILE_EXTENSION) {
        if mode == CliMode::Test {
            crate::cli_test::validate_explicit_test_file(&path)?;
        }
        return Ok(CliInput::SourceFile(path));
    }
    if has_extension(&path, PROJECT_FILE_EXTENSION) {
        return Ok(CliInput::ProjectFile(path));
    }
    if matches!(mode, CliMode::Check | CliMode::Test)
        && has_extension(&path, WORKSPACE_FILE_EXTENSION)
    {
        return Ok(CliInput::WorkspaceFile(path));
    }

    let expected = match mode {
        CliMode::Run => "a `.fpas` or `.fpasprj` file",
        CliMode::Check => "a `.fpas`, `.fpasprj`, or `.fpasworkspace` file",
        CliMode::Test => "a `.fpas` file, directory, `.fpasprj`, or `.fpasworkspace` file",
    };
    Err(format!(
        "Unsupported input `{}`. Expected {expected}.",
        path.display()
    ))
}

fn discover_input(cwd: &Path, mode: CliMode) -> Result<CliInput, String> {
    match mode {
        CliMode::Check | CliMode::Test => discover_check_input(cwd),
        CliMode::Run => discover_run_input(cwd),
    }
}

fn discover_run_input(cwd: &Path) -> Result<CliInput, String> {
    if let Some(workspace_path) = discover_workspace_file(cwd)? {
        let program_path = discover_run_project_in_workspace(&workspace_path)?;
        return Ok(CliInput::ProjectFile(program_path));
    }

    discover_project_file(cwd)
}

fn discover_check_input(cwd: &Path) -> Result<CliInput, String> {
    if let Some(workspace_path) = discover_workspace_file(cwd)? {
        return Ok(CliInput::WorkspaceFile(workspace_path));
    }

    discover_project_file(cwd)
}

fn normalize_input_path(input: &str, cwd: &Path) -> PathBuf {
    let path = PathBuf::from(input);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

fn discover_project_file(cwd: &Path) -> Result<CliInput, String> {
    let read_dir = fs::read_dir(cwd)
        .map_err(|e| format!("Error reading current directory `{}`: {e}", cwd.display()))?;

    let mut candidates = Vec::<PathBuf>::new();
    for entry in read_dir {
        let entry = entry.map_err(|e| {
            format!(
                "Error reading entries from current directory `{}`: {e}",
                cwd.display()
            )
        })?;
        let path = entry.path();
        if path.is_file() && has_extension(&path, PROJECT_FILE_EXTENSION) {
            candidates.push(path);
        }
    }

    candidates.sort();

    match candidates.len() {
        0 => Err(format!(
            "No `.fpasprj` file found in current directory `{}`.\n  help: Pass a `.fpas`, `.fpasprj`, or `.fpasworkspace` path explicitly.",
            cwd.display()
        )),
        1 => Ok(CliInput::ProjectFile(candidates.remove(0))),
        _ => {
            let entries = candidates
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!(
                "Found multiple `.fpasprj` files in current directory `{}`: {entries}.\n  help: Pass the desired `.fpasprj` file path explicitly.",
                cwd.display()
            ))
        }
    }
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}
