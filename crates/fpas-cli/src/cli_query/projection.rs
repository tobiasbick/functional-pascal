//! JSON projection of checked facts; all resolution and visibility remain in the core.

use super::{protocol::*, report};
use fpas_language_service::{
    DocumentSnapshot, SourceCandidate, SourceFindResult, SourceQueryOperation, SourceQueryResult,
    SourceQueryTarget,
};
use std::path::Path;

pub(super) fn target(target: &SourceQueryTarget, cwd: &Path) -> Target {
    Target {
        name: target.name.clone(),
        qualified_name: target.qualified_name.clone(),
        kind: target.kind.map_or("builtin".into(), |kind| {
            format!("{kind:?}").to_ascii_lowercase()
        }),
        origin: if target.snapshot.is_some() {
            "source"
        } else {
            "builtin"
        }
        .into(),
        declaration: target
            .snapshot
            .as_ref()
            .zip(target.symbol.as_ref())
            .and_then(|(s, t)| report::location(s, t.selection_span, cwd)),
        owner: target.target_ref.as_ref().and_then(|r| r.owner.clone()),
        contexts: Vec::new(),
        visibility: target.symbol.as_ref().map(|s| s.visibility),
        detail: target.symbol.as_ref().map(|s| s.detail.clone()),
        documentation: None,
        checked: target.target_ref.is_some() || target.snapshot.is_none(),
        target_ref: target.target_ref.clone(),
        reference_unavailable: target.reference_unavailable.map(str::to_owned),
    }
}

fn candidate(candidate: &SourceCandidate, cwd: &Path) -> Target {
    let mut projected = target(&candidate.target, cwd);
    projected.origin = serde_json::to_value(candidate.origin)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default();
    projected.owner = candidate.owner.clone();
    projected.contexts = candidate.contexts.clone();
    projected.documentation = candidate.documentation.clone();
    projected.detail = Some(candidate.detail.clone());
    projected.checked = candidate.checked;
    projected
}

pub(super) fn find(report: &mut Report, result: SourceFindResult, cwd: &Path) {
    let symbols = result
        .symbols
        .iter()
        .map(|s| candidate(s, cwd))
        .collect::<Vec<_>>();
    report.coverage = Coverage {
        analysis_complete: result.reasons.iter().all(|r| *r == "result_limit"),
        ..Coverage::default()
    };
    report.coverage.collections.insert(
        "symbols".into(),
        Collection {
            returned: symbols.len(),
            known_total: result.known_total,
            complete: result.known_total == Some(symbols.len()),
        },
    );
    report.result = Some(Payload::Find { symbols });
    report.outcome = Outcome::Ok;
    report.exit_code = 0;
    report.diagnostics = result
        .diagnostics
        .iter()
        .map(|d| report::fp_diagnostic(d, cwd))
        .collect();
    for reason in result.reasons {
        report::incomplete(report, reason);
    }
}

pub(super) fn query(
    report: &mut Report,
    result: SourceQueryResult,
    selected: Option<&DocumentSnapshot>,
    operation: SourceQueryOperation,
    cwd: &Path,
) {
    report.diagnostics = result
        .diagnostics
        .iter()
        .map(|d| report::fp_diagnostic(d, cwd))
        .collect();
    report.target = result.target.as_ref().map(|t| target(t, cwd));
    report.candidates = result
        .candidates
        .iter()
        .map(|s| candidate(s, cwd))
        .collect();
    let mut locations = result
        .matches
        .iter()
        .filter_map(|m| report::location(&m.snapshot, m.span, cwd))
        .collect::<Vec<_>>();
    locations.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.byte_span.start.cmp(&b.byte_span.start))
    });
    let count = locations.len();
    let type_info = result
        .expression_type
        .as_ref()
        .map(|ty| TypeInfo {
            display: ty.type_name.clone(),
            kind: ty.kind.into(),
            location: selected.and_then(|s| report::location(s, ty.span, cwd)),
            declaration: ty
                .declaration
                .as_ref()
                .and_then(|d| report::location(&d.snapshot, d.span, cwd)),
        })
        .or_else(|| {
            result.declaration_type.as_ref().map(|ty| TypeInfo {
                display: ty.display.clone(),
                kind: ty.kind.into(),
                location: None,
                declaration: None,
            })
        });
    let found = match operation {
        SourceQueryOperation::Callers | SourceQueryOperation::Callees => result.calls.is_some(),
        SourceQueryOperation::Inspect => result.inspection.is_some(),
        SourceQueryOperation::References => result.target.is_some(),
        SourceQueryOperation::Definition => !locations.is_empty(),
        SourceQueryOperation::Type => type_info.is_some(),
    };
    let mut invalid_location = count != result.matches.len()
        || report
            .target
            .as_ref()
            .is_some_and(|t| t.origin == "source" && t.declaration.is_none())
        || result.expression_type.is_some()
            && type_info.as_ref().is_some_and(|t| t.location.is_none());
    report.result = Some(match operation {
        SourceQueryOperation::Callers | SourceQueryOperation::Callees => {
            let calls = result
                .calls
                .as_ref()
                .map(|c| c.project(|s, span| report::location(s, span, cwd).ok_or(())))
                .transpose()
                .unwrap_or_else(|()| {
                    invalid_location = true;
                    None
                })
                .unwrap_or_default();
            if operation == SourceQueryOperation::Callers {
                Payload::Callers { calls }
            } else {
                Payload::Callees { calls }
            }
        }
        SourceQueryOperation::Inspect => {
            let inspection = result
                .inspection
                .as_ref()
                .map(|inspection| {
                    inspection.project(|s, span| report::location(s, span, cwd).ok_or(()))
                })
                .transpose()
                .unwrap_or_else(|()| {
                    invalid_location = true;
                    None
                })
                .unwrap_or_default();
            Payload::Inspect { inspection }
        }
        SourceQueryOperation::References => Payload::Refs { locations },
        SourceQueryOperation::Definition => Payload::Definition { locations },
        SourceQueryOperation::Type => Payload::Type {
            unavailable_reason: type_info.is_none().then(|| "no_value_type".into()),
            r#type: type_info,
        },
    });
    report.outcome = if found {
        Outcome::Ok
    } else {
        Outcome::NotFound
    };
    report.exit_code = if found { 0 } else { 1 };
    report.coverage = Coverage {
        analysis_complete: true,
        ..Coverage::default()
    };
    if matches!(
        operation,
        SourceQueryOperation::Definition | SourceQueryOperation::References
    ) {
        report.coverage.collections.insert(
            "locations".into(),
            Collection {
                returned: count,
                known_total: (!result.reasons.contains("result_limit")).then_some(count),
                complete: true,
            },
        );
    }
    if let Some(total) = result.candidate_total {
        report.coverage.collections.insert(
            "candidates".into(),
            Collection {
                returned: report.candidates.len(),
                known_total: Some(total),
                complete: report.candidates.is_empty() && found || report.candidates.len() == total,
            },
        );
    }
    if let Some(reason) = result.selection_error {
        if reason == "ambiguous_target" {
            report.outcome = Outcome::Error;
        }
        report::diagnostic(
            report,
            reason,
            "The exact name does not select one eligible declaration.",
        );
    } else if !found && result.reasons.is_empty() {
        let reason = if result.target.as_ref().is_some_and(|t| t.snapshot.is_none()) {
            "no_source_definition"
        } else {
            "no_target"
        };
        report::diagnostic(
            report,
            reason,
            "No source construct satisfying this operation was selected.",
        );
    }
    for reason in result.reasons {
        report::incomplete(report, reason);
    }
    if invalid_location {
        report::incomplete(report, "invalid_location");
    }
    if let Some(Payload::Callers { calls } | Payload::Callees { calls }) = &report.result {
        for (name, collection) in &calls.collections {
            report.coverage.collections.insert(
                name.clone(),
                Collection {
                    returned: collection.returned,
                    known_total: collection.known_total,
                    complete: collection.complete,
                },
            );
        }
    }
}
