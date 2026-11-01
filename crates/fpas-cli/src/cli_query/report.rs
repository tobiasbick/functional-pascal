//! Bounded schema-2 reports; exact identities and edit bytes are never text-clipped.

use super::protocol::*;
use fpas_diagnostics::{FileDiagnostic, SourceSpan};
use fpas_language_service::DocumentSnapshot;
use serde_json::{Value, json};
use std::path::Path;

pub(super) const MAX_JSON: usize = 8 * 1024 * 1024;

pub(super) fn path(path: &Path, cwd: &Path) -> String {
    path.strip_prefix(cwd)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(super) fn empty(args: &[String], _cwd: &Path) -> Report {
    let operation = args
        .get(1)
        .filter(|s| {
            [
                "find",
                "inspect",
                "refs",
                "definition",
                "type",
                "callers",
                "callees",
            ]
            .contains(&s.as_str())
                || args.first().is_some_and(|s| s == "refactor")
                    && ["rename", "extract-variable", "organize-imports", "apply"]
                        .contains(&s.as_str())
        })
        .cloned();
    let result = match operation.as_deref() {
        Some("callers") => Some(Payload::Callers {
            calls: Default::default(),
        }),
        Some("callees") => Some(Payload::Callees {
            calls: Default::default(),
        }),
        Some("find") => Some(Payload::Find {
            symbols: Vec::new(),
        }),
        Some("refs") => Some(Payload::Refs {
            locations: Vec::new(),
        }),
        Some("definition") => Some(Payload::Definition {
            locations: Vec::new(),
        }),
        Some("type") => Some(Payload::Type {
            r#type: None,
            unavailable_reason: Some("state_unavailable".into()),
        }),
        _ => None,
    };
    let mut limits = Limits::default();
    if matches!(operation.as_deref(), Some("refs" | "callers" | "callees")) {
        limits.result_count = Some(1000);
    }
    if operation.as_deref() == Some("find") {
        limits.result_count = Some(100);
    }
    Report {
        schema_version: 2,
        operation,
        outcome: Outcome::Error,
        exit_code: 1,
        request: Request::default(),
        context: None,
        snapshot: None,
        target: None,
        candidates: Vec::new(),
        result,
        diagnostics: Vec::new(),
        limits,
        coverage: Coverage {
            reasons: ["state_unavailable".into()].into(),
            ..Coverage::default()
        },
    }
}

pub(super) fn diagnostic(report: &mut Report, code: &str, message: &str) {
    report
        .diagnostics
        .push(json!({"kind":"query","code":code,"severity":"error","message":message}));
}

pub(super) fn incomplete(report: &mut Report, reason: &str) {
    report.outcome = Outcome::Incomplete;
    report.exit_code = 3;
    report.coverage.reasons.insert(reason.into());
    if !["result_limit", "json_limit", "dynamic_targets"].contains(&reason) {
        report.coverage.analysis_complete = false;
        if let Some(Payload::Callers { calls } | Payload::Callees { calls }) = &mut report.result {
            calls.direct_edges_complete = false;
            calls.runtime_targets_complete = false;
            for collection in calls.collections.values_mut() {
                collection.known_total = None;
                collection.complete = false;
            }
        }
        if let Some(target) = &mut report.target
            && !["body_unavailable", "unsupported_target"].contains(&reason)
        {
            target.target_ref = None;
            target.reference_unavailable = Some("analysis_incomplete".into());
        }
        for target in &mut report.candidates {
            target.target_ref = None;
            target.reference_unavailable = Some("analysis_incomplete".into());
        }
        if let Some(Payload::Find { symbols }) = &mut report.result {
            for target in symbols {
                target.target_ref = None;
                target.reference_unavailable = Some("analysis_incomplete".into());
            }
        }
    }
    for collection in report.coverage.collections.values_mut() {
        collection.complete = false;
        if !["result_limit", "json_limit", "dynamic_targets"].contains(&reason) {
            collection.known_total = None;
        }
    }
}

pub(super) fn failure(args: &[String], cwd: &Path, code: &str, message: &str, exit: i32) -> Report {
    let mut report = empty(args, cwd);
    report.exit_code = exit;
    diagnostic(&mut report, code, message);
    report
}

pub(super) fn fp_diagnostic(record: &FileDiagnostic, cwd: &Path) -> Value {
    let source_path = record.path.as_deref().map(|p| path(p, cwd));
    fpas_diagnostics::render_json(
        source_path.as_deref(),
        record.source.as_deref(),
        &record.diagnostic,
    )
    .ok()
    .and_then(|s| serde_json::from_str(&s).ok())
    .unwrap_or_else(
        || json!({"code":record.diagnostic.code.to_string(),"message":record.diagnostic.message}),
    )
}

pub(super) fn location(
    snapshot: &DocumentSnapshot,
    span: SourceSpan,
    cwd: &Path,
) -> Option<Location> {
    let position = |offset| {
        let pos = snapshot.line_index().position(offset)?;
        let start = snapshot.line_index().line_start(pos.line)?;
        Some(Position {
            line: pos.line + 1,
            column: snapshot.source().get(start..offset)?.chars().count() + 1,
        })
    };
    let start = position(span.offset())?;
    let end = position(span.end())?;
    let pos = snapshot.line_index().position(span.offset())?;
    let line = snapshot.line_index().line_range(pos.line)?;
    let text = snapshot.source().get(line.clone())?;
    let column = snapshot
        .source()
        .get(line.start..span.offset())?
        .chars()
        .count();
    let skip = column.saturating_sub(60);
    let preview = text.chars().skip(skip).take(160).collect::<String>();
    Some(Location {
        path: path(snapshot.path(), cwd),
        range: Range { start, end },
        byte_span: ByteSpan {
            start: span.offset(),
            length: span.length(),
        },
        preview: Preview {
            text: preview,
            column_start: skip + 1,
            truncated: skip > 0 || text.chars().count() > skip + 160,
        },
    })
}

fn clip_diagnostics(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            let mut clipped = Vec::new();
            for (name, value) in fields.iter_mut() {
                if ["message", "expected", "found", "hint", "title"].contains(&name.as_str())
                    && let Some(text) = value.as_str()
                    && text.chars().count() > 4096
                {
                    *value = json!(text.chars().take(4096).collect::<String>());
                    clipped.push(format!("{name}_truncated"));
                } else {
                    clip_diagnostics(value);
                }
            }
            for name in clipped {
                fields.insert(name, json!(true));
            }
        }
        Value::Array(values) => {
            for value in values {
                clip_diagnostics(value);
            }
        }
        _ => {}
    }
}

pub(super) fn finish(report: &mut Report) -> Vec<u8> {
    let mut clipped = false;
    let mut clip_target = |target: &mut Target| {
        for text in [&mut target.detail, &mut target.documentation]
            .into_iter()
            .flatten()
        {
            if text.chars().count() > 4096 {
                *text = text.chars().take(4096).collect();
                clipped = true;
            }
        }
    };
    if let Some(target) = &mut report.target {
        clip_target(target);
    }
    for target in &mut report.candidates {
        clip_target(target);
    }
    if let Some(Payload::Find { symbols }) = &mut report.result {
        for target in symbols {
            clip_target(target);
        }
    }
    if clipped {
        report.coverage.omissions.push("declaration_text".into());
    }
    for diagnostic in &mut report.diagnostics {
        clip_diagnostics(diagnostic);
    }
    if let Some(Payload::Type {
        r#type: Some(ty), ..
    }) = &mut report.result
        && ty.display.chars().count() > 4096
    {
        ty.display = ty.display.chars().take(4096).collect();
        report.coverage.omissions.push("type.display".into());
    }
    loop {
        if let Ok(mut bytes) = serde_json::to_vec(report) {
            bytes.push(b'\n');
            if bytes.len() <= MAX_JSON {
                return bytes;
            }
        }
        incomplete(report, "json_limit");
        if matches!(report.result, Some(Payload::Refactor { .. })) {
            report.result = None;
            report.outcome = Outcome::Error;
            report.exit_code = 1;
            report.coverage.reasons.insert("plan_limit".into());
            diagnostic(
                report,
                "plan_limit",
                "The exact plan exceeds the public JSON limit; no source was published.",
            );
            continue;
        }
        if let Some(Payload::Callers { calls } | Payload::Callees { calls }) = &mut report.result
            && let Some(name) = calls.trim_for_budget()
        {
            if let Some(collection) = calls.collections.get(name) {
                report.coverage.collections.insert(
                    name.into(),
                    Collection {
                        returned: collection.returned,
                        known_total: collection.known_total,
                        complete: false,
                    },
                );
            }
            continue;
        }
        if let Some(Payload::Find { symbols }) = &mut report.result
            && symbols.len() > 1
        {
            symbols.truncate(symbols.len().div_ceil(2));
            if let Some(coverage) = report.coverage.collections.get_mut("symbols") {
                coverage.returned = symbols.len();
            }
            continue;
        }
        if report.candidates.len() > 1 {
            report
                .candidates
                .truncate(report.candidates.len().div_ceil(2));
            if let Some(coverage) = report.coverage.collections.get_mut("candidates") {
                coverage.returned = report.candidates.len();
            }
            continue;
        }
        if let Some(locations) = report.result.as_mut().and_then(Payload::locations_mut)
            && locations.len() > 1
        {
            locations.truncate(locations.len().div_ceil(2));
            if let Some(coverage) = report.coverage.collections.get_mut("locations") {
                coverage.returned = locations.len();
            }
            continue;
        }
        if report.diagnostics.len() > 1 {
            report
                .diagnostics
                .truncate(report.diagnostics.len().div_ceil(2));
            continue;
        }
        // Required identities can be larger than the ceiling. Drop the complete payload,
        // never truncate an identifier or leave an unbounded retry loop.
        report.request = Request::default();
        report.context = None;
        report.snapshot = None;
        report.target = None;
        report.candidates.clear();
        report.result = None;
        report.coverage.collections.clear();
        report.coverage.reasons = ["json_limit".into()].into();
        if report.operation.as_ref().is_some_and(|o| o.len() > 64) {
            report.operation = None;
        }
        report.coverage.omissions = vec!["payload".into()];
        report.diagnostics.clear();
        diagnostic(
            report,
            "json_limit",
            "The exact result exceeds the public JSON limit.",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oversized_required_fields_fail_with_bounded_json_instead_of_partial_identity() {
        let mut report = empty(&["query".into(), "refs".into()], Path::new("/workspace"));
        report.request.selector = Some(Selector::Position {
            path: "x".repeat(MAX_JSON),
            line: 1,
            column: 1,
        });
        let bytes = finish(&mut report);
        assert!(bytes.len() <= MAX_JSON);
        assert_eq!(report.outcome, Outcome::Incomplete);
        assert!(report.request.selector.is_none());
        assert_eq!(report.diagnostics[0]["code"], "json_limit");
        assert!(serde_json::from_slice::<Value>(&bytes).is_ok());
    }
}
