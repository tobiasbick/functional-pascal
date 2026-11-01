//! Non-interactive checked preview and explicit plan application arguments.

mod help;
use super::{HelpTopic, QuerySelector, ResolvedCli};
use crate::cli_paths::{has_extension, normalize_input_path};
pub(super) use help::text as help;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RefactorCliConfig {
    pub action: RefactorAction,
    pub context: Option<PathBuf>,
    pub standard_library: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub timeout: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RefactorAction {
    AddParameter {
        selector: QuerySelector,
        name: String,
        r#type: String,
        mode: fpas_language_service::ParameterMode,
        argument: String,
    },
    Rename {
        selector: QuerySelector,
        name: String,
    },
    ExtractVariable {
        path: PathBuf,
        start: (usize, usize),
        end: (usize, usize),
    },
    OrganizeImports {
        path: PathBuf,
    },
    Apply {
        plan: PathBuf,
    },
}

pub(super) fn resolve(args: &[String], cwd: &Path) -> Result<ResolvedCli, String> {
    if args
        .iter()
        .any(|a| ["--version", "-V"].contains(&a.as_str()))
    {
        return Ok(ResolvedCli::Version);
    }
    let command = args
        .first()
        .map(String::as_str)
        .ok_or("Missing refactoring action; use fpas refactor --help.")?;
    let topic = match command {
        "--help" | "-h" if args.len() == 1 => return Ok(ResolvedCli::Help(HelpTopic::Refactor)),
        "rename" => HelpTopic::RefactorRename,
        "add-parameter" => HelpTopic::RefactorAddParameter,
        "extract-variable" => HelpTopic::RefactorExtract,
        "organize-imports" => HelpTopic::RefactorImports,
        "apply" => HelpTopic::RefactorApply,
        _ => {
            return Err(format!(
                "Unknown refactoring action `{command}`; use fpas refactor --help."
            ));
        }
    };
    if matches!(&args[1..], [arg] if arg == "--help" || arg == "-h") {
        return Ok(ResolvedCli::Help(topic));
    }
    let mut options = BTreeMap::new();
    let mut positional = None;
    let mut arguments = args[1..].iter();
    while let Some(arg) = arguments.next() {
        if arg.starts_with('-') {
            let allowed = arg == "--timeout"
                || command != "apply"
                    && ["--context", "--std-lib", "--out"].contains(&arg.as_str())
                || match command {
                    "rename" => ["--symbol", "--target-ref", "--name"].contains(&arg.as_str()),
                    "add-parameter" => [
                        "--symbol",
                        "--target-ref",
                        "--name",
                        "--type",
                        "--mode",
                        "--argument",
                    ]
                    .contains(&arg.as_str()),
                    "extract-variable" => arg == "--range",
                    "apply" => arg == "--plan",
                    _ => false,
                };
            if !allowed {
                return Err(format!(
                    "Unknown option `{arg}` for fpas refactor {command}."
                ));
            }
            let value = arguments
                .next()
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| format!("{arg} requires a value."))?;
            if options.insert(arg.as_str(), value.as_str()).is_some() {
                return Err(format!("Repeated {arg}."));
            }
        } else if positional.replace(arg.as_str()).is_some() {
            return Err("Expected exactly one source selector.".into());
        }
    }
    let context = options
        .get("--context")
        .map(|p| normalize_input_path(p, cwd));
    if context
        .as_ref()
        .is_some_and(|p| !has_extension(p, "fpasprj") && !has_extension(p, "fpasworkspace"))
    {
        return Err("--context requires a .fpasprj or .fpasworkspace manifest.".into());
    }
    let timeout = options
        .get("--timeout")
        .map(|s| {
            s.parse::<u64>()
                .ok()
                .filter(|n| *n > 0)
                .ok_or("--timeout requires positive seconds.")
        })
        .transpose()?
        .unwrap_or(30);
    let required = |name| {
        options
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing {name}."))
    };
    let action = match command {
        "rename" | "add-parameter" => {
            if usize::from(positional.is_some())
                + usize::from(options.contains_key("--symbol"))
                + usize::from(options.contains_key("--target-ref"))
                != 1
            {
                return Err("Choose exactly one position, --symbol or --target-ref.".into());
            }
            let selector = if let Some(name) = options.get("--symbol") {
                if context.is_none() {
                    return Err("Name selection requires --context <manifest>.".into());
                }
                QuerySelector::Symbol((*name).into())
            } else if let Some(path) = options.get("--target-ref") {
                QuerySelector::Reference(normalize_input_path(path, cwd))
            } else {
                super::query::parse_position(positional.ok_or("Missing source position.")?, cwd)?
            };
            if command == "add-parameter" {
                RefactorAction::AddParameter {
                    selector,
                    name: required("--name")?.into(),
                    r#type: required("--type")?.into(),
                    mode: match required("--mode")? {
                        "value" => fpas_language_service::ParameterMode::Value,
                        "var" => fpas_language_service::ParameterMode::Var,
                        _ => return Err("--mode requires value or var.".into()),
                    },
                    argument: required("--argument")?.into(),
                }
            } else {
                RefactorAction::Rename {
                    selector,
                    name: required("--name")?.into(),
                }
            }
        }
        "extract-variable" if positional.is_none() => {
            let mut parts = required("--range")?.rsplitn(5, ':');
            let number = |part: Option<&str>| {
                part.and_then(|p| p.parse::<usize>().ok())
                    .filter(|n| *n > 0)
                    .ok_or("Invalid --range coordinates.")
            };
            let end_column = number(parts.next())?;
            let end_line = number(parts.next())?;
            let start_column = number(parts.next())?;
            let start_line = number(parts.next())?;
            let path = normalize_input_path(
                parts
                    .next()
                    .filter(|p| !p.is_empty())
                    .ok_or("Missing --range source path.")?,
                cwd,
            );
            if !has_extension(&path, "fpas") || (start_line, start_column) >= (end_line, end_column)
            {
                return Err("--range requires a nonempty half-open .fpas source range.".into());
            }
            RefactorAction::ExtractVariable {
                path,
                start: (start_line, start_column),
                end: (end_line, end_column),
            }
        }
        "organize-imports" => {
            let path = normalize_input_path(positional.ok_or("Missing source file.")?, cwd);
            if !has_extension(&path, "fpas") {
                return Err("Import organization requires a .fpas source.".into());
            }
            RefactorAction::OrganizeImports { path }
        }
        "apply" if positional.is_none() => RefactorAction::Apply {
            plan: normalize_input_path(required("--plan")?, cwd),
        },
        _ => return Err("Unexpected positional argument.".into()),
    };
    Ok(ResolvedCli::Refactor(RefactorCliConfig {
        action,
        context,
        timeout,
        standard_library: options
            .get("--std-lib")
            .map(|p| normalize_input_path(p, cwd)),
        output: options.get("--out").map(|p| normalize_input_path(p, cwd)),
    }))
}
