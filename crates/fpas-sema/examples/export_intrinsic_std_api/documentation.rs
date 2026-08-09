//! Markdown documentation rendering for generated intrinsic declarations.

use std::collections::HashMap;
use std::fmt::Write as _;

use fpas_sema::{IntrinsicStdSymbol, IntrinsicStdSymbolKind, Ty};

use super::DocumentationRow;

pub(super) fn render_documentation(
    output: &mut String,
    unit: &str,
    name: &str,
    symbol: &IntrinsicStdSymbol,
    documentation: &HashMap<(String, String), DocumentationRow>,
    indent: &str,
) {
    let row = documentation_row(unit, name, documentation);
    let summary = row
        .map(|row| row.summary.as_str())
        .filter(|summary| !summary.is_empty())
        .map_or_else(|| fallback_summary(name, symbol.kind), sentence_summary);
    let _ = writeln!(output, "{indent}// {summary}");
    let parameters = match &symbol.ty {
        Ty::Function(function) => &function.params,
        Ty::Procedure(procedure) => &procedure.params,
        _ => return,
    };
    let documented_parameters = if parameters.is_empty() {
        row.and_then(|row| parameter_names(&row.signature))
            .unwrap_or_default()
    } else {
        parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect()
    };
    if documented_parameters.is_empty() {
        return;
    }
    let _ = writeln!(output, "{indent}//");
    let _ = writeln!(output, "{indent}// Parameters:");
    for parameter in documented_parameters {
        let parameter_type = parameters
            .iter()
            .find(|candidate| candidate.name.eq_ignore_ascii_case(&parameter))
            .map(|candidate| candidate.ty.to_string());
        let _ = writeln!(
            output,
            "{indent}// - `{parameter}`: {}",
            parameter_description(&parameter, name, parameter_type.as_deref())
        );
    }
}

pub(super) fn parameter_description(parameter: &str, routine: &str, ty: Option<&str>) -> String {
    let description = match parameter.to_ascii_lowercase().as_str() {
        "path" => "File or directory path processed by the operation",
        "pattern" => "Glob or matching pattern evaluated by the operation",
        "text" | "s" => "UTF-8 text processed by the operation",
        "name" => "Name looked up or processed by the operation",
        "index" => "Zero-based index unless the operation documents another base",
        "start" => "Zero-based starting position",
        "len" => "Number of elements or characters to process",
        "count" => "Number of values to create or process",
        "milliseconds" => "Non-negative duration in milliseconds",
        "command" => "Executable name or path to start",
        "args" => "Command-line arguments in invocation order",
        "app" => "Graphics application session receiving the operation",
        "width" => "Width in cells or pixels as documented by the operation",
        "height" => "Height in cells or pixels as documented by the operation",
        "x" | "x1" | "x2" | "centerx" => "Horizontal coordinate",
        "y" | "y1" | "y2" | "centery" => "Vertical coordinate",
        "color" | "fg" | "bg" => "Color value used by the operation",
        "value" | "default" | "expected" | "actual" => {
            "Value consumed or returned by the operation"
        }
        "f" => "Callback invoked by the higher-order operation",
        "handle" => "Task handle whose result is consumed",
        "tasks" => "Task handles to wait for",
        "key" => "Lookup key or keyboard event, according to the operation",
        "cond" => "Boolean condition checked by the operation",
        "msg" => "Diagnostic message reported by the operation",
        "line" => "Input line queued for the next read",
        "title" => "Window title displayed by the host",
        "pixels" => "Row-major pixel buffer uploaded to the host",
        "parts" | "segments" => "Values combined in their existing order",
        "items" => "Array elements stored in the constructed value",
        "delim" => "Delimiter used to split or join text",
        "old" => "Text to replace",
        "new" => "Replacement text",
        _ => {
            return ty.map_or_else(
                || {
                    format!(
                        "Input value used by `{}`.",
                        routine.rsplit('.').next().unwrap_or(routine)
                    )
                },
                |ty| {
                    format!(
                        "`{ty}` input value used by `{}`.",
                        routine.rsplit('.').next().unwrap_or(routine)
                    )
                },
            );
        }
    };
    format!("{description}.")
}

pub(super) fn documentation_row<'a>(
    unit: &str,
    name: &str,
    documentation: &'a HashMap<(String, String), DocumentationRow>,
) -> Option<&'a DocumentationRow> {
    documentation.get(&(unit.to_owned(), name.to_ascii_lowercase()))
}

fn parameter_names(signature: &str) -> Option<Vec<String>> {
    let start = signature.find('(')? + 1;
    let end = signature.rfind(')')?;
    let value = signature.get(start..end)?;
    if value.trim().is_empty() || value.trim() == "..." {
        return Some(Vec::new());
    }
    Some(
        value
            .split(';')
            .filter_map(|parameter| {
                let before_type = parameter.split(':').next()?.trim();
                let name = before_type
                    .trim_start_matches("mutable ")
                    .split(',')
                    .next_back()?
                    .trim();
                (!name.is_empty()).then(|| name.to_owned())
            })
            .collect(),
    )
}

fn sentence_summary(summary: &str) -> String {
    let mut result = summary.trim().trim_end_matches('.').to_owned();
    if let Some(first) = result.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    result.push('.');
    result
}

fn fallback_summary(name: &str, kind: IntrinsicStdSymbolKind) -> String {
    let short_name = name.rsplit('.').next().unwrap_or(name);
    match kind {
        IntrinsicStdSymbolKind::Constant => format!("Intrinsic `{short_name}` constant."),
        IntrinsicStdSymbolKind::Function => format!("Runs the intrinsic `{short_name}` function."),
        IntrinsicStdSymbolKind::Procedure => {
            format!("Runs the intrinsic `{short_name}` procedure.")
        }
        IntrinsicStdSymbolKind::Type => format!("Intrinsic `{short_name}` data type."),
        IntrinsicStdSymbolKind::EnumMember => format!("`{short_name}` enum member."),
    }
}

pub(super) fn default_value(ty: &Ty) -> &'static str {
    match ty {
        Ty::Real => "0.0",
        Ty::Boolean => "false",
        Ty::String => "''",
        _ => "0",
    }
}
