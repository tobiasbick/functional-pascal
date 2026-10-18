//! Argument parsing for `fpas doc`.
//!
//! Spec: [Looking up declarations](../../../docs/pascal/program-structure/cli.md#looking-up-declarations).

use std::path::Path;

use crate::cli_paths::normalize_input_path;

use super::mode::{CliMode, usage_error};
use super::{DocCliConfig, DocReportFormat, HelpTopic, ResolvedCli};

/// Resolves the looked-up name, the optional project path and output options.
pub(super) fn resolve_doc_cli(args: &[String], cwd: &Path) -> Result<ResolvedCli, String> {
    let usage = usage_error(CliMode::Doc);
    let mut positional = Vec::new();
    let mut report = None;
    let mut standard_library = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "-h" | "--help" if args.len() == 1 => return Ok(ResolvedCli::Help(HelpTopic::Doc)),
            "--report" => {
                let value = take_value(args, &mut index, "--report")?;
                if value != "json" {
                    return Err(format!(
                        "Unsupported report format `{value}`.\n  help: `fpas doc string.TrimLeft --report json`."
                    ));
                }
                if report.replace(DocReportFormat::Json).is_some() {
                    return Err("Duplicate `--report` option.".to_string());
                }
            }
            "--std-lib" => {
                let value = take_value(args, &mut index, "--std-lib")?;
                if standard_library
                    .replace(normalize_input_path(value, cwd))
                    .is_some()
                {
                    return Err("Duplicate `--std-lib` option.".to_string());
                }
            }
            arg if arg.starts_with('-') => {
                return Err(format!("Unknown option `{arg}` for `fpas doc`.\n{usage}"));
            }
            arg => positional.push(arg.to_string()),
        }
        index += 1;
    }
    let (query, input) = match positional.as_slice() {
        [query] => (query.clone(), cwd.to_path_buf()),
        [query, path] => (query.clone(), normalize_input_path(path, cwd)),
        [] => return Err(format!("Missing name to look up.\n{usage}")),
        _ => {
            return Err(format!(
                "`fpas doc` takes one name and at most one path.\n{usage}"
            ));
        }
    };
    if query.trim().is_empty() {
        return Err(format!("The name to look up cannot be empty.\n{usage}"));
    }
    Ok(ResolvedCli::Doc(DocCliConfig {
        query,
        input,
        cwd: cwd.to_path_buf(),
        standard_library,
        report,
    }))
}

fn take_value<'a>(args: &'a [String], index: &mut usize, option: &str) -> Result<&'a str, String> {
    let usage = usage_error(CliMode::Doc);
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .filter(|value| !value.starts_with('-'))
        .ok_or_else(|| format!("`{option}` requires a value.\n{usage}"))
}
