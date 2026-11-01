//! LSP projections retain shared identity and coverage while converting source positions.

use super::{invalid, model::*};
use crate::documents::DocumentRequestError;
use fpas_language_service::{
    DocumentSnapshot, SourceCandidate, SourceFindResult, SourceQueryResult, SourceQueryTarget,
};
use tower_lsp_server::ls_types::Uri;

pub(super) fn find(
    response: &mut SourceQueryResponse,
    result: SourceFindResult,
) -> Result<(), DocumentRequestError> {
    let symbols = result
        .symbols
        .iter()
        .map(candidate)
        .collect::<Result<Vec<_>, _>>()?;
    response.outcome = if result.reasons.is_empty() {
        "ok"
    } else {
        "incomplete"
    };
    response.coverage.analysis_complete = result.reasons.iter().all(|r| *r == "result_limit")
        && response.context.policy == fpas_language_service::QueryPolicy::CheckedContext;
    response.coverage.collections.insert(
        "symbols".into(),
        SourceCollection {
            returned: symbols.len(),
            known_total: result.known_total,
            complete: result.known_total == Some(symbols.len()),
        },
    );
    response.coverage.reasons = result.reasons.iter().map(|r| (*r).into()).collect();
    response.result = SourcePayload::Find { symbols };
    response.diagnostics = diagnostics(&result.diagnostics);
    Ok(())
}

pub(super) fn query(
    response: &mut SourceQueryResponse,
    result: SourceQueryResult,
    selected: Option<&DocumentSnapshot>,
    operation: SourceOperation,
) -> Result<(), DocumentRequestError> {
    let locations = result
        .matches
        .iter()
        .map(|m| location(&m.snapshot, m.span))
        .collect::<Result<Vec<_>, _>>()?;
    let count = locations.len();
    let type_info = result
        .expression_type
        .as_ref()
        .map(|ty| {
            Ok::<_, DocumentRequestError>(SourceType {
                display: ty.type_name.clone(),
                kind: ty.kind.into(),
                location: Some(location(
                    selected.ok_or_else(|| invalid("Missing type snapshot."))?,
                    ty.span,
                )?),
                declaration: ty
                    .declaration
                    .as_ref()
                    .map(|d| location(&d.snapshot, d.span))
                    .transpose()?,
            })
        })
        .transpose()?
        .or_else(|| {
            result.declaration_type.as_ref().map(|ty| SourceType {
                display: ty.display.clone(),
                kind: ty.kind.into(),
                location: None,
                declaration: None,
            })
        });
    let found = match operation {
        SourceOperation::Callers | SourceOperation::Callees => result.calls.is_some(),
        SourceOperation::Inspect => result.inspection.is_some(),
        SourceOperation::Refs => result.target.is_some(),
        SourceOperation::Definition => !locations.is_empty(),
        SourceOperation::Type => type_info.is_some(),
        SourceOperation::Find => return Err(invalid("Expected a declaration query.")),
    };
    response.result = match operation {
        SourceOperation::Callers | SourceOperation::Callees => {
            let calls = result
                .calls
                .as_ref()
                .map(|c| c.project(location))
                .transpose()?
                .unwrap_or_default();
            for (name, collection) in &calls.collections {
                response.coverage.collections.insert(
                    name.clone(),
                    SourceCollection {
                        returned: collection.returned,
                        known_total: collection.known_total,
                        complete: collection.complete,
                    },
                );
            }
            if operation == SourceOperation::Callers {
                SourcePayload::Callers { calls }
            } else {
                SourcePayload::Callees { calls }
            }
        }
        SourceOperation::Inspect => SourcePayload::Inspect {
            inspection: result
                .inspection
                .as_ref()
                .map(|i| i.project(location))
                .transpose()?
                .unwrap_or_default(),
        },
        SourceOperation::Refs => SourcePayload::Refs { locations },
        SourceOperation::Definition => SourcePayload::Definition { locations },
        SourceOperation::Type => SourcePayload::Type {
            unavailable_reason: type_info.is_none().then_some("no_value_type"),
            r#type: type_info,
        },
        SourceOperation::Find => return Err(invalid("Expected a declaration query.")),
    };
    response.target = result.target.as_ref().map(target).transpose()?;
    response.candidates = result
        .candidates
        .iter()
        .map(candidate)
        .collect::<Result<_, _>>()?;
    response.diagnostics = diagnostics(&result.diagnostics);
    response.coverage.analysis_complete = result
        .reasons
        .iter()
        .all(|r| matches!(*r, "result_limit" | "dynamic_targets"));
    response.coverage.reasons = result.reasons.iter().map(|r| (*r).into()).collect();
    response.outcome = if !result.reasons.is_empty() {
        "incomplete"
    } else if result.selection_error == Some("ambiguous_target") {
        "error"
    } else if found {
        "ok"
    } else {
        "not_found"
    };
    if matches!(
        operation,
        SourceOperation::Definition | SourceOperation::Refs
    ) {
        response.coverage.collections.insert(
            "locations".into(),
            SourceCollection {
                returned: count,
                known_total: result.reasons.is_empty().then_some(count),
                complete: result.reasons.is_empty(),
            },
        );
    }
    if result.candidate_total.is_some() || !response.candidates.is_empty() {
        response.coverage.collections.insert(
            "candidates".into(),
            SourceCollection {
                returned: response.candidates.len(),
                known_total: result.candidate_total,
                complete: result.candidate_total == Some(response.candidates.len()) || found,
            },
        );
    }
    if let Some(code) = result.selection_error {
        response.diagnostics.push(serde_json::json!({"kind":"query","code":code,"message":"The exact name does not select one eligible declaration."}));
    }
    Ok(())
}

fn candidate(candidate: &SourceCandidate) -> Result<SourceTarget, DocumentRequestError> {
    let mut target = target(&candidate.target)?;
    target.owner = candidate.owner.clone();
    target.contexts = candidate.contexts.clone();
    target.checked = candidate.checked;
    target.origin = serde_json::to_value(candidate.origin)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default();
    target.detail = Some(candidate.detail.clone());
    target.documentation = candidate.documentation.clone();
    Ok(target)
}

fn target(target: &SourceQueryTarget) -> Result<SourceTarget, DocumentRequestError> {
    Ok(SourceTarget {
        name: target.name.clone(),
        qualified_name: target.qualified_name.clone(),
        kind: target.kind,
        declaration: target
            .snapshot
            .as_ref()
            .zip(target.symbol.as_ref())
            .map(|(s, t)| location(s, t.selection_span))
            .transpose()?,
        target_ref: target.target_ref.clone(),
        reference_unavailable: target.reference_unavailable,
        owner: target.target_ref.as_ref().and_then(|r| r.owner.clone()),
        contexts: Vec::new(),
        visibility: target.symbol.as_ref().map(|s| s.visibility),
        origin: if target.snapshot.is_some() {
            "source"
        } else {
            "builtin"
        }
        .into(),
        detail: target.symbol.as_ref().map(|s| s.detail.clone()),
        documentation: None,
        checked: target.target_ref.is_some() || target.snapshot.is_none(),
    })
}

fn location(
    snapshot: &DocumentSnapshot,
    span: fpas_diagnostics::SourceSpan,
) -> Result<SourceLocation, DocumentRequestError> {
    Ok(SourceLocation {
        uri: Uri::from_file_path(snapshot.path())
            .ok_or_else(|| invalid("Source path cannot be represented as a URI."))?,
        range: crate::navigation::span_range(snapshot, span)
            .map_err(|_| invalid("Invalid source range."))?,
        byte_start: span.offset(),
        byte_length: span.length(),
    })
}

pub(crate) fn diagnostics(records: &[fpas_diagnostics::FileDiagnostic]) -> Vec<serde_json::Value> {
    records.iter().map(|d| {
        let path = d.path.as_ref().map(|p| p.to_string_lossy());
        fpas_diagnostics::render_json(path.as_deref(), d.source.as_deref(), &d.diagnostic).ok()
            .and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_else(|| serde_json::json!({"code":d.diagnostic.code.to_string(),"message":d.diagnostic.message}))
    }).collect()
}
