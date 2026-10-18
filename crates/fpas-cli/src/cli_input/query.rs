//! Arguments and focused help for read-only source queries.

use std::path::{Path, PathBuf};

use super::{HelpTopic, ResolvedCli};
use crate::cli_paths::{has_extension, normalize_input_path};

const USAGE: &str = "Usage: fpas query <refs | def | type> <file.fpas:line:column> [--project <path>] [--std-lib <dir>]\n  help: `fpas query refs src/main.fpas:12:5` or `fpas query --help`.";

/// One protocol-independent language-service question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueryKind {
    Refs,
    Definition,
    Type,
}

impl QueryKind {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Refs => "refs",
            Self::Definition => "def",
            Self::Type => "type",
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
    pub include_declaration: bool,
}

pub(super) fn resolve(args: &[String], cwd: &Path) -> Result<ResolvedCli, String> {
    let Some(command) = args.first() else {
        return Err(format!("Missing query kind.\n{USAGE}"));
    };
    let (kind, topic) = match command.as_str() {
        "--help" | "-h" if args.len() == 1 => return Ok(ResolvedCli::Help(HelpTopic::Query)),
        "refs" => (QueryKind::Refs, HelpTopic::QueryRefs),
        "def" => (QueryKind::Definition, HelpTopic::QueryDef),
        "type" => (QueryKind::Type, HelpTopic::QueryType),
        other => return Err(format!("Unknown query kind `{other}`.\n{USAGE}")),
    };
    if matches!(&args[1..], [arg] if arg == "--help" || arg == "-h") {
        return Ok(ResolvedCli::Help(topic));
    }
    let mut position = None;
    let mut project = None;
    let mut standard_library = None;
    let mut include_declaration = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--project" | "--std-lib" => {
                let option = &args[index];
                index += 1;
                let value = args
                    .get(index)
                    .filter(|value| !value.is_empty() && !value.starts_with('-'))
                    .ok_or_else(|| format!("`{option}` requires a path.\n{USAGE}"))?;
                let slot = if option == "--project" {
                    &mut project
                } else {
                    &mut standard_library
                };
                if slot.replace(normalize_input_path(value, cwd)).is_some() {
                    return Err(format!("Duplicate `{option}` option.\n{USAGE}"));
                }
            }
            "--include-declaration" if kind == QueryKind::Refs && !include_declaration => {
                include_declaration = true;
            }
            other if other.starts_with('-') => {
                return Err(format!(
                    "Unknown or repeated option `{other}` for `fpas query {}`.\n{USAGE}",
                    kind.name()
                ));
            }
            value => {
                if position.replace(value).is_some() {
                    return Err(format!("Expected exactly one source position.\n{USAGE}"));
                }
            }
        }
        index += 1;
    }
    let position = position.ok_or_else(|| format!("Missing source position.\n{USAGE}"))?;
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
    Ok(ResolvedCli::Query(QueryCliConfig {
        kind,
        path,
        line,
        column,
        project,
        standard_library,
        include_declaration,
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
        HelpTopic::QueryRefs => {
            "Find project references to the symbol at a source position as JSON.

Usage:
  fpas query refs <file.fpas:line:column> [--include-declaration] [--project <path>] [--std-lib <dir>]

Options:
  --include-declaration  Include the defining declaration (default: usages only)
  --project <path>       Explicit project, workspace, or folder context
  --std-lib <dir>        Replace the complete source standard library
  -h, --help             Print this help

Positions use one-based lines and Unicode-scalar columns. No source is changed.

Examples:
  fpas query refs src/main.fpas:12:5
  fpas query refs src/math.fpas:4:17 --include-declaration --project suite.fpasworkspace
"
        }
        HelpTopic::QueryDef => {
            "Find the defining declaration of the symbol at a source position as JSON.

Usage:
  fpas query def <file.fpas:line:column> [--project <path>] [--std-lib <dir>]

Options:
  --project <path>  Explicit project, workspace, or folder context
  --std-lib <dir>   Replace the complete source standard library
  -h, --help        Print this help

Positions use one-based lines and Unicode-scalar columns. No source is changed.

Examples:
  fpas query def src/main.fpas:12:5
  fpas query def src/main.fpas:12:5 --project app.fpasprj
"
        }
        HelpTopic::QueryType => {
            "Find the checked type of the smallest expression at a source position as JSON.

Usage:
  fpas query type <file.fpas:line:column> [--project <path>] [--std-lib <dir>]

Options:
  --project <path>  Explicit project, workspace, or folder context
  --std-lib <dir>   Replace the complete source standard library
  -h, --help        Print this help

Positions use one-based lines and Unicode-scalar columns. No source is changed.

Examples:
  fpas query type src/main.fpas:12:5
  fpas query type src/main.fpas:12:5 --std-lib ./lib
"
        }
        _ => {
            "Query source navigation through the language service; one JSON document on stdout.

Usage:
  fpas query <refs | def | type> <file.fpas:line:column> [options]

Commands:
  refs  Find usages of a resolved symbol (optionally include its declaration)
  def   Find the resolved declaration
  type  Find the checked type of the smallest enclosing expression

Run `fpas query <command> --help` for options. Lines and Unicode-scalar columns
start at 1. Empty matches succeed; source, context and argument failures exit 1.

Examples:
  fpas query refs src/main.fpas:12:5
  fpas query def src/main.fpas:12:5
  fpas query type src/main.fpas:12:5
"
        }
    }
}
