//! Arguments and focused help for source discovery, inspection and navigation.

use std::path::{Path, PathBuf};

use super::{HelpTopic, ResolvedCli};
use crate::cli_paths::{has_extension, normalize_input_path};

const USAGE: &str = "Usage: fpas query <find | inspect | refs | definition | type | callers | callees> <selector> [options]\n  help: `fpas query find Clamp --context app.fpasprj` or `fpas query --help`.";

/// One protocol-independent language-service question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueryKind {
    Find,
    Inspect,
    Callers,
    Callees,
    Refs,
    Definition,
    Type,
}

impl QueryKind {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Find => "find",
            Self::Inspect => "inspect",
            Self::Callers => "callers",
            Self::Callees => "callees",
            Self::Refs => "refs",
            Self::Definition => "definition",
            Self::Type => "type",
        }
    }
}

/// Fully resolved inputs for one JSON query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueryCliConfig {
    pub kind: QueryKind,
    pub selector: QuerySelector,
    pub project: Option<PathBuf>,
    pub standard_library: Option<PathBuf>,
    pub timeout: u64,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum QuerySelector {
    Position {
        path: PathBuf,
        line: usize,
        column: usize,
    },
    Symbol(String),
    Reference(PathBuf),
    Search(String),
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
        "find" => (QueryKind::Find, HelpTopic::QueryFind),
        "inspect" => (QueryKind::Inspect, HelpTopic::QueryInspect),
        "callers" => (QueryKind::Callers, HelpTopic::QueryCallers),
        "callees" => (QueryKind::Callees, HelpTopic::QueryCallees),
        "definition" => (QueryKind::Definition, HelpTopic::QueryDef),
        "type" => (QueryKind::Type, HelpTopic::QueryType),
        other => return Err(format!("Unknown query kind `{other}`.\n{USAGE}")),
    };
    if matches!(&args[1..], [arg] if arg == "--help" || arg == "-h") {
        return Ok(ResolvedCli::Help(topic));
    }
    let mut position = None;
    let mut selector = None;
    let mut project = None;
    let mut standard_library = None;
    let mut timeout = None;
    let mut limit = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--symbol" | "--target-ref" if kind != QueryKind::Find => {
                let option = &args[index];
                index += 1;
                let value = args
                    .get(index)
                    .filter(|s| !s.trim().is_empty() && !s.starts_with('-'))
                    .ok_or_else(|| format!("`{option}` requires a value.\n{USAGE}"))?;
                let selected = if option == "--symbol" {
                    QuerySelector::Symbol(value.clone())
                } else {
                    QuerySelector::Reference(normalize_input_path(value, cwd))
                };
                if selector.replace(selected).is_some() {
                    return Err(format!(
                        "Select exactly one position, symbol or target reference.\n{USAGE}"
                    ));
                }
            }
            "--context" | "--std-lib" => {
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
            "--timeout" | "--limit" => {
                let option = &args[index];
                if option == "--limit"
                    && !matches!(
                        kind,
                        QueryKind::Refs | QueryKind::Find | QueryKind::Callers | QueryKind::Callees
                    )
                {
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
            other if other.starts_with('-') => {
                return Err(format!(
                    "Unknown or repeated option `{other}` for `fpas query {}`.\n{USAGE}",
                    kind.name()
                ));
            }
            value => {
                if position.is_none() {
                    position = Some(value);
                } else {
                    return Err(format!("Expected exactly one source position.\n{USAGE}"));
                }
            }
        }
        index += 1;
    }
    if selector.is_some() && position.is_some() {
        return Err(format!(
            "Select exactly one position, symbol or target reference.\n{USAGE}"
        ));
    }
    let selector = match (selector, position) {
        (Some(selector), None) => selector,
        (None, Some(text)) if kind == QueryKind::Find && !text.trim().is_empty() => {
            QuerySelector::Search(text.into())
        }
        (None, Some(text)) if kind != QueryKind::Find => parse_position(text, cwd)?,
        _ => return Err(format!("Missing search text or source selector.\n{USAGE}")),
    };
    if matches!(
        selector,
        QuerySelector::Search(_) | QuerySelector::Symbol(_)
    ) && project.is_none()
    {
        return Err("Name search and selection require --context <manifest>.".into());
    }
    if let Some(context) = &project
        && !(has_extension(context, "fpasprj") || has_extension(context, "fpasworkspace"))
    {
        return Err("--context requires a .fpasprj or .fpasworkspace manifest.".into());
    }
    Ok(ResolvedCli::Query(QueryCliConfig {
        kind,
        selector,
        project,
        standard_library,
        timeout: timeout.unwrap_or(30),
        limit: limit.unwrap_or(if kind == QueryKind::Find { 100 } else { 1000 }) as usize,
    }))
}

pub(super) fn parse_position(position: &str, cwd: &Path) -> Result<QuerySelector, String> {
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
    Ok(QuerySelector::Position { path, line, column })
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
        HelpTopic::QueryCallers => {
            "Find checked incoming calls and callable uses as one JSON result.\n\nUsage:\n  fpas query callers <selector> [--context <manifest>] [--std-lib <dir>] [--timeout <seconds>] [--limit <count>]\n\nSelectors: file.fpas:line:column, --symbol <name>, or --target-ref <file>.\nName selection requires a manifest. Defaults: 30 seconds, 1000 per collection; maximum 10000.\nDirect calls, callable values, callback arguments and dynamic calls are separate.\nUnresolved calls are conservatively included from the selected context.\nCheck direct_edges_complete and runtime_targets_complete; no direct edges does not prove no executions.\nExits: 0 complete answer, 1 invalid request or no target, 3 incomplete coverage.\n\nExamples:\n  fpas query callers --symbol App.Math.Clamp --context app.fpasprj\n  fpas query callers --target-ref clamp.target.json --limit 20\n"
        }
        HelpTopic::QueryCallees => {
            "Find checked outgoing calls in one selected source body as JSON.\n\nUsage:\n  fpas query callees <selector> [--context <manifest>] [--std-lib <dir>] [--timeout <seconds>] [--limit <count>]\n\nSelectors: file.fpas:line:column, --symbol <name>, or --target-ref <file>.\nName selection requires a manifest. Defaults: 30 seconds, 1000 per collection; maximum 10000.\nNested routines, closures and record defaults have their own owners.\nNative bodies are unavailable; dynamic targets remain explicit.\nExits: 0 complete answer (including an empty body), 1 invalid request or no target, 3 incomplete coverage.\n\nExamples:\n  fpas query callees src/math.fpas:4:17 --context app.fpasprj\n  fpas query callees --symbol App.Math.Clamp --context app.fpasprj\n"
        }
        HelpTopic::QueryInspect => {
            "Inspect a checked declaration, its implementation and directly named types.\n\nUsage:\n  fpas query inspect <file.fpas:line:column | --symbol <name> | --target-ref <file>> [--context <manifest>] [--std-lib <dir>] [--timeout <seconds>]\n\nName selection requires a manifest. Default timeout: 30 seconds.\nFixed budgets: 32 KiB of exact source excerpts and 12 supporting types.\nGenerated intrinsic stubs are declarations; native implementations are unavailable.\n\nExamples:\n  fpas query inspect --symbol App.Math.Clamp --context app.fpasprj\n  fpas query inspect --target-ref clamp.target.json\n  fpas query inspect src/math.fpas:4:17\n"
        }
        HelpTopic::QueryFind => {
            "Find checked declarations as one JSON result.\n\nUsage:\n  fpas query find <text> --context <manifest> [--std-lib <dir>] [--timeout <seconds>] [--limit <count>]\n\nDefaults: timeout 30 seconds, limit 100 symbols (maximum 10000).\nRanks exact names before partial matches; reports omissions and analysis coverage.\nExits: 0 complete answer (including no matches), 1 invalid input, 3 incomplete coverage.\n\nExamples:\n  fpas query find Clamp --context app.fpasprj\n  fpas query find Queue --context suite.fpasworkspace --limit 20\n"
        }
        HelpTopic::QueryRefs => {
            "Find resolved source uses as one JSON result.\n\nUsage:\n  fpas query refs <selector> [--context <manifest>] [--std-lib <dir>] [--timeout <seconds>] [--limit <count>]\n\nSelectors: file.fpas:line:column, --symbol <name>, or --target-ref <file>.\nName selection requires a manifest; ambiguous names return candidates.\nDefaults: timeout 30 seconds, limit 1000 uses (maximum 10000).\nThe declaration is separate under target. Positions are one-based Unicode scalars.\nExits: 0 complete success, 1 invalid request or no target, 3 incomplete coverage.\n\nExamples:\n  fpas query refs src/main.fpas:12:5\n  fpas query refs --symbol App.Math.Clamp --context app.fpasprj\n  fpas query refs --target-ref clamp.target.json --limit 200\n"
        }
        HelpTopic::QueryDef => {
            "Find the resolved source declaration as one JSON result.\n\nUsage:\n  fpas query definition <selector> [--context <manifest>] [--std-lib <dir>] [--timeout <seconds>]\n\nSelectors: file.fpas:line:column, --symbol <name>, or --target-ref <file>.\nName selection requires a manifest; ambiguous names return candidates.\nThe default deadline is 30 seconds. Positions are one-based Unicode scalars.\nExits: 0 complete success, 1 invalid request or no definition, 3 incomplete coverage.\n\nExamples:\n  fpas query definition src/main.fpas:12:5\n  fpas query definition --symbol App.Math.Clamp --context app.fpasprj\n  fpas query definition --target-ref clamp.target.json\n"
        }
        HelpTopic::QueryType => {
            "Find a checked static type as one JSON result.\n\nUsage:\n  fpas query type <selector> [--context <manifest>] [--std-lib <dir>] [--timeout <seconds>]\n\nSelectors: file.fpas:line:column, --symbol <name>, or --target-ref <file>.\nName selection requires a manifest and selects the declaration's value type.\nThe default deadline is 30 seconds. Positions are one-based Unicode scalars.\nExits: 0 complete success, 1 invalid request or no value type, 3 incomplete coverage.\n\nExamples:\n  fpas query type src/main.fpas:12:5\n  fpas query type --symbol App.Math.Clamp --context app.fpasprj\n  fpas query type --target-ref clamp.target.json\n"
        }
        _ => {
            "Query saved sources through the language service.\n\nUsage:\n  fpas query <find | inspect | refs | definition | type | callers | callees> <selector> [options]\n\nCommands:\n  find        Search checked declarations in an explicit manifest context\n  inspect     Read the signature, implementation and direct supporting types\n  refs        Find resolved uses; the declaration is separate under target\n  definition  Find the defining identifier\n  type        Read a checked expression or declaration value type\n  callers     Read incoming direct calls and other callable uses\n  callees     Read outgoing calls in one source body\n\nSelectors: file.fpas:line:column, --symbol <name>, or --target-ref <file>.\nFind takes search text. Name search/selection requires --context <manifest>.\nEvery request emits one schema-2 JSON result, including failures.\nUse fpas refactor for checked source changes. Run each command with --help.\n\nExamples:\n  fpas query find Clamp --context app.fpasprj\n  fpas query inspect --symbol App.Math.Clamp --context app.fpasprj\n  fpas query callers --target-ref clamp.target.json\n  fpas query type src/main.fpas:12:5\n"
        }
    }
}
