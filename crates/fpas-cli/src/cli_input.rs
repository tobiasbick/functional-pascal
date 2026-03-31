use std::fs;
use std::path::{Path, PathBuf};

use fpas_lexer::DefineSet;

const SOURCE_FILE_EXTENSION: &str = "fpas";
const PROJECT_FILE_EXTENSION: &str = "fpasprj";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CliInput {
    SourceFile(PathBuf),
    ProjectFile(PathBuf),
}

#[derive(Debug, Clone)]
pub(crate) struct CliConfig {
    pub input: CliInput,
    pub defines: DefineSet,
}

pub(crate) fn resolve_cli_input(args: &[String], cwd: &Path) -> Result<CliInput, String> {
    if args.len() > 1 {
        return Err("Usage: fpas [<file.fpas | file.fpasprj>]".to_string());
    }

    Ok(resolve_cli_config(args, cwd)?.input)
}

pub(crate) fn resolve_cli_config(args: &[String], cwd: &Path) -> Result<CliConfig, String> {
    let mut defines = DefineSet::new();
    let mut input = None::<String>;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        if arg == "-D" || arg == "--define" {
            let Some(name) = args.get(index + 1) else {
                return Err(
                    "Missing name after `--define`.\n  help: Use `-D DEBUG` or `--define DEBUG`."
                        .to_string(),
                );
            };
            add_define(&mut defines, name)?;
            index += 2;
            continue;
        }

        if let Some(name) = arg.strip_prefix("--define=") {
            add_define(&mut defines, name)?;
            index += 1;
            continue;
        }

        if let Some(name) = arg.strip_prefix("-D") {
            if !name.is_empty() {
                add_define(&mut defines, name)?;
                index += 1;
                continue;
            }
        }

        if arg.starts_with('-') {
            return Err(format!(
                "Unknown option `{arg}`.\n  help: Use `-D NAME` / `--define NAME` or pass a `.fpas` / `.fpasprj` path."
            ));
        }

        if input.replace(arg.clone()).is_some() {
            return Err(
                "Usage: fpas [-D NAME | --define NAME]... [<file.fpas | file.fpasprj>]".to_string(),
            );
        }
        index += 1;
    }

    let input = match input {
        Some(input) => resolve_explicit_input(&input, cwd),
        None => discover_project_file(cwd),
    }?;

    Ok(CliConfig { input, defines })
}

fn add_define(defines: &mut DefineSet, raw_name: &str) -> Result<(), String> {
    let name = raw_name.trim();
    if name.is_empty() {
        return Err(
            "Conditional symbol names must not be empty.\n  help: Use `-D DEBUG` or `--define RELEASE`."
                .to_string(),
        );
    }

    defines.define(name);
    Ok(())
}

fn resolve_explicit_input(input: &str, cwd: &Path) -> Result<CliInput, String> {
    let path = normalize_input_path(input, cwd);
    if has_extension(&path, SOURCE_FILE_EXTENSION) {
        return Ok(CliInput::SourceFile(path));
    }
    if has_extension(&path, PROJECT_FILE_EXTENSION) {
        return Ok(CliInput::ProjectFile(path));
    }

    Err(format!(
        "Unsupported input `{}`. Expected a `.fpas` or `.fpasprj` file.",
        path.display()
    ))
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
            "No `.fpasprj` file found in current directory `{}`.\n  help: Pass a `.fpas` or `.fpasprj` path explicitly.",
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
