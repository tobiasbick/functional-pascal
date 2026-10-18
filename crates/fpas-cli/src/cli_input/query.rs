//! Arguments and focused help for source navigation and rename.

use std::path::{Path, PathBuf};

use super::{HelpTopic, ResolvedCli};
use crate::cli_paths::{has_extension, normalize_input_path};

const USAGE: &str = "Usage: fpas query <refs | definition | type | rename> <file.fpas:line:column> [<new-name>] [options]\n  help: `fpas query rename src/main.fpas:12:5 NewName --dry-run` or `fpas query --help`.";

/// One protocol-independent language-service question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueryKind {
    Refs,
    Definition,
    Type,
    Rename,
}

impl QueryKind {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Refs => "refs",
            Self::Definition => "definition",
            Self::Type => "type",
            Self::Rename => "rename",
        }
    }
}

/// Fully resolved inputs for one JSON query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueryCliConfig {
    pub kind: QueryKind,
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub project: Option<PathBuf>,
    pub standard_library: Option<PathBuf>,
    pub timeout: u64,
    pub limit: usize,
    pub new_name: Option<String>,
    pub dry_run: bool,
}

pub(super) fn resolve(args: &[String], cwd: &Path) -> Result<ResolvedCli, String> {
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        return Ok(ResolvedCli::Version);
    }
    let Some(command) = args.first() else {
        return Err(format!("Missing query kind.\n{USAGE}"));
    };
    let (kind, topic) = match command.as_str() {
        "--help" | "-h" if args.len() == 1 => return Ok(ResolvedCli::Help(HelpTopic::Query)),
        "refs" => (QueryKind::Refs, HelpTopic::QueryRefs),
        "definition" => (QueryKind::Definition, HelpTopic::QueryDef),
        "type" => (QueryKind::Type, HelpTopic::QueryType),
        "rename" => (QueryKind::Rename, HelpTopic::QueryRename),
        other => return Err(format!("Unknown query kind `{other}`.\n{USAGE}")),
    };
    if matches!(&args[1..], [arg] if arg == "--help" || arg == "-h") {
        return Ok(ResolvedCli::Help(topic));
    }
    let mut position = None;
    let mut project = None;
    let mut standard_library = None;
    let mut timeout = None;
    let mut limit = None;
    let mut new_name = None;
    let mut dry_run = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--context" | "--project" | "--std-lib" => {
                if (args[index] == "--project" && kind != QueryKind::Rename)
                    || (args[index] == "--context" && kind == QueryKind::Rename)
                {
                    return Err(format!("Unknown option `{}`.\n{USAGE}", args[index]));
                }
                let option = &args[index];
                index += 1;
                let value = args
                    .get(index)
                    .filter(|value| !value.is_empty() && !value.starts_with('-'))
                    .ok_or_else(|| format!("`{option}` requires a path.\n{USAGE}"))?;
                let slot = if option != "--std-lib" {
                    &mut project
                } else {
                    &mut standard_library
                };
                if slot.replace(normalize_input_path(value, cwd)).is_some() {
                    return Err(format!("Duplicate `{option}` option.\n{USAGE}"));
                }
            }
            "--timeout" | "--limit" if kind != QueryKind::Rename => {
                let option = &args[index];
                if option == "--limit" && kind != QueryKind::Refs {
                    return Err(format!("Unknown option `{option}`.\n{USAGE}"));
                }
                index += 1;
                let number = args
                    .get(index)
                    .and_then(|s| positive_number(Some(s)))
                    .ok_or_else(|| format!("`{option}` requires a positive integer.\n{USAGE}"))?
                    as u64;
                let slot = if option == "--timeout" {
                    &mut timeout
                } else {
                    &mut limit
                };
                if slot.replace(number).is_some() {
                    return Err(format!("Duplicate `{option}` option.\n{USAGE}"));
                }
                if option == "--limit" && number > 10000 {
                    return Err("--limit cannot exceed 10000.".into());
                }
            }
            "--dry-run" if kind == QueryKind::Rename && !dry_run => dry_run = true,
            other if other.starts_with('-') => {
                return Err(format!(
                    "Unknown or repeated option `{other}` for `fpas query {}`.\n{USAGE}",
                    kind.name()
                ));
            }
            value => {
                if position.is_none() {
                    position = Some(value);
                } else if kind == QueryKind::Rename && new_name.is_none() {
                    new_name = Some(value.to_owned());
                } else {
                    return Err(format!("Expected exactly one source position.\n{USAGE}"));
                }
            }
        }
        index += 1;
    }
    let position = position.ok_or_else(|| format!("Missing source position.\n{USAGE}"))?;
    if kind == QueryKind::Rename && new_name.is_none() {
        return Err(format!("Missing replacement name for rename.\n{USAGE}"));
    }
    // Split from the right so Windows drive letters remain part of the path.
    let mut parts = position.rsplitn(3, ':');
    let column = positive_number(parts.next());
    let line = positive_number(parts.next());
    let path = parts.next().filter(|path| !path.is_empty());
    let (Some(path), Some(line), Some(column)) = (path, line, column) else {
        return Err(format!(
            "Invalid source position `{position}`; use file.fpas:line:column with positive numbers.\n{USAGE}"
        ));
    };
    let path = normalize_input_path(path, cwd);
    if !has_extension(&path, "fpas") {
        return Err(format!(
            "Query positions require a `.fpas` source file.\n{USAGE}"
        ));
    }
    if kind != QueryKind::Rename
        && let Some(context) = &project
        && !(has_extension(context, "fpasprj") || has_extension(context, "fpasworkspace"))
    {
        return Err("--context requires a .fpasprj or .fpasworkspace manifest.".into());
    }
    Ok(ResolvedCli::Query(QueryCliConfig {
        kind,
        path,
        line,
        column,
        project,
        standard_library,
        timeout: timeout.unwrap_or(30),
        limit: limit.unwrap_or(1000) as usize,
        new_name,
        dry_run,
    }))
}

fn positive_number(value: Option<&str>) -> Option<usize> {
    value
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))?
        .parse()
        .ok()
        .filter(|value| *value > 0)
}

pub(super) const fn help(topic: HelpTopic) -> &'static str {
    match topic {
        HelpTopic::QueryRename => {
            "Safely rename a resolved declaration and its project references; report edits as JSON.

Usage:
  fpas query rename <file.fpas:line:column> <new-name> [--dry-run] [--project <path>] [--std-lib <dir>]

Options:
  --dry-run         Validate and list edits without writing sources
  --project <path>  Explicit project, workspace, or folder context
  --std-lib <dir>   Replace the complete source standard library
  -h, --help        Print this help

Without --dry-run, applies edits to saved sources. Rejects invalid identifiers,
name conflicts, erroneous sources, and edits outside the selected root or in Std.
Positions use one-based lines and Unicode-scalar columns.

Examples:
  fpas query rename src/main.fpas:12:5 Total --dry-run
  fpas query rename src/math.fpas:4:17 Compute --project suite.fpasworkspace
"
        }
        HelpTopic::QueryRefs => "Find resolved source uses as one JSON result.\n\nUsage:\n  fpas query refs <file.fpas:line:column> [--context <manifest>] [--std-lib <dir>] [--timeout <seconds>] [--limit <count>]\n\nDefaults: timeout 30 seconds, limit 1000 uses (maximum 10000).\nThe declaration is separate under target. Positions are one-based Unicode scalars.\nExits: 0 complete success, 1 invalid request or no target, 3 incomplete coverage.\n\nExamples:\n  fpas query refs src/main.fpas:12:5\n  fpas query refs src/math.fpas:4:17 --context suite.fpasworkspace --limit 200\n",
        HelpTopic::QueryDef => "Find the resolved source declaration as one JSON result.\n\nUsage:\n  fpas query definition <file.fpas:line:column> [--context <manifest>] [--std-lib <dir>] [--timeout <seconds>]\n\nThe default deadline is 30 seconds. Positions are one-based Unicode scalars.\nExits: 0 complete success, 1 invalid request or no definition, 3 incomplete coverage.\n\nExamples:\n  fpas query definition src/main.fpas:12:5\n  fpas query definition src/main.fpas:12:5 --context app.fpasprj\n",
        HelpTopic::QueryType => "Find the checked static type at a source position as one JSON result.\n\nUsage:\n  fpas query type <file.fpas:line:column> [--context <manifest>] [--std-lib <dir>] [--timeout <seconds>]\n\nThe default deadline is 30 seconds. Positions are one-based Unicode scalars.\nExits: 0 complete success, 1 invalid request or no value type, 3 incomplete coverage.\n\nExamples:\n  fpas query type src/main.fpas:12:5\n  fpas query type src/main.fpas:12:5 --std-lib ./lib\n",
        _ => "Query saved sources through the language service.\n\nUsage:\n  fpas query <refs | definition | type | rename> <file.fpas:line:column> [options]\n\nCommands:\n  refs        Find resolved uses; the declaration is separate under target\n  definition  Find the defining identifier\n  type        Inspect a checked expression or value binding\n  rename <new-name>  Rename a symbol; --dry-run lists edits without writing\n\nRun fpas query <command> --help for options. Navigation emits one JSON result\nincluding failures; rename retains its separate edit/publication contract.\n\nExamples:\n  fpas query refs src/main.fpas:12:5\n  fpas query definition src/main.fpas:12:5\n  fpas query type src/main.fpas:12:5\n  fpas query rename src/main.fpas:12:5 Total --dry-run\n",
    }
}
