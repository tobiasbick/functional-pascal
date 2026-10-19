//! Trace coverage shared by execution, setup errors, and report clipping.

use serde_json::{Value, json};

/// Retains the requested selectors before binding, including early setup failures.
pub fn request(report: &mut Value, selectors: &[String], limit: usize) {
    report["limits"]["trace_events"] = json!(limit);
    report["trace"]["requested"] = json!(selectors);
    if !selectors.is_empty() {
        report["trace"]["complete"] = json!(false);
        report["trace"]["reasons"] = json!(["state_unavailable"]);
        report["trace"]["total_count"] = Value::Null;
    }
}

/// Propagates lost requested history after snapshot capture has set its own coverage.
pub fn coverage(report: &mut Value) {
    if report["trace"]["complete"] == false {
        report["capture"]["incomplete"] = json!(true);
        let reasons = report["trace"]["reasons"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        for reason in reasons {
            append_reason(&mut report["capture"]["reasons"], reason);
        }
        if report["exit_code"] == 0 {
            report["exit_code"] = json!(3);
        }
    }
}

/// Removes one whole tail event, retaining sequence and known total counts.
pub(super) fn shrink(report: &mut Value) -> usize {
    let Some(event) = report["trace"]["events"].as_array_mut().and_then(Vec::pop) else {
        return 0;
    };
    let removed = super::report::encoded_size(&event);
    report["trace"]["event_count"] =
        json!(report["trace"]["events"].as_array().map_or(0, Vec::len));
    report["trace"]["complete"] = json!(false);
    append_reason(&mut report["trace"]["reasons"], json!("byte_limit"));
    coverage(report);
    removed
}

pub(super) fn append_reason(reasons: &mut Value, reason: Value) {
    if let Some(reasons) = reasons.as_array_mut()
        && !reasons.contains(&reason)
    {
        reasons.push(reason);
    }
}
