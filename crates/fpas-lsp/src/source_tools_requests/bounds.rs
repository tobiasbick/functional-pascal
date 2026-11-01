//! Public rich replies have the same text and JSON ceilings as CLI source queries.

use super::model::*;
use crate::documents::DocumentRequestError;
use fpas_language_service::SourceQueryError;

pub(super) fn apply(response: &mut SourceQueryResponse) -> Result<(), DocumentRequestError> {
    let mut clipped = false;
    let mut target = |target: &mut SourceTarget| {
        for text in [&mut target.detail, &mut target.documentation]
            .into_iter()
            .flatten()
        {
            clipped |= clip(text);
        }
    };
    if let Some(value) = &mut response.target {
        target(value);
    }
    for candidate in &mut response.candidates {
        target(candidate);
    }
    if let SourcePayload::Find { symbols } = &mut response.result {
        for symbol in symbols {
            target(symbol);
        }
    }
    if let SourcePayload::Type {
        r#type: Some(ty), ..
    } = &mut response.result
    {
        clipped |= clip(&mut ty.display);
    }
    if clipped {
        response.coverage.omissions.push("declaration_text".into());
    }
    for diagnostic in &mut response.diagnostics {
        clip_diagnostic(diagnostic);
    }
    loop {
        if serde_json::to_vec(response).is_ok_and(|bytes| bytes.len() <= 8 * 1024 * 1024) {
            return Ok(());
        }
        response.outcome = "incomplete";
        response.coverage.reasons.insert("json_limit".into());
        let collection = match &mut response.result {
            SourcePayload::Callers { calls } | SourcePayload::Callees { calls } => calls
                .trim_for_budget()
                .and_then(|name| calls.collections.get(name).map(|c| (name, c.returned))),
            SourcePayload::Find { symbols } if symbols.len() > 1 => {
                symbols.truncate(symbols.len().div_ceil(2));
                Some(("symbols", symbols.len()))
            }
            SourcePayload::Refs { locations } | SourcePayload::Definition { locations }
                if locations.len() > 1 =>
            {
                locations.truncate(locations.len().div_ceil(2));
                Some(("locations", locations.len()))
            }
            _ if response.candidates.len() > 1 => {
                response
                    .candidates
                    .truncate(response.candidates.len().div_ceil(2));
                Some(("candidates", response.candidates.len()))
            }
            _ => None,
        };
        if let Some((name, returned)) = collection {
            if let Some(coverage) = response.coverage.collections.get_mut(name) {
                coverage.returned = returned;
                coverage.complete = false;
            }
            continue;
        }
        if response.diagnostics.len() > 1 {
            response
                .diagnostics
                .truncate(response.diagnostics.len().div_ceil(2));
            continue;
        }
        return Err(SourceQueryError {
            code: "json_limit",
            message: "The exact source-tool reply exceeds the public JSON limit.".into(),
            diagnostics: Vec::new(),
        }
        .into());
    }
}

fn clip(text: &mut String) -> bool {
    if text.chars().count() <= 4096 {
        return false;
    }
    *text = text.chars().take(4096).collect();
    true
}

fn clip_diagnostic(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            let mut truncated = Vec::new();
            for (name, value) in fields.iter_mut() {
                if ["message", "expected", "found", "hint", "title"].contains(&name.as_str())
                    && let serde_json::Value::String(text) = value
                {
                    if clip(text) {
                        truncated.push(format!("{name}_truncated"));
                    }
                } else {
                    clip_diagnostic(value);
                }
            }
            for name in truncated {
                fields.insert(name, serde_json::Value::Bool(true));
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                clip_diagnostic(value);
            }
        }
        _ => {}
    }
}
