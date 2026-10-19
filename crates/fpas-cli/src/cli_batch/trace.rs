//! Parent-owned recovery of bounded trace request metadata on unpublished results.

use serde_json::{Value, json};

/// Adds known argv metadata to errors before full parsing, binding or publication.
/// Parsed worker results already contain their canonical requests and watch set.
pub(super) fn preserve_request(report: &mut Value, args: &[String]) {
    let args = args
        .iter()
        .take_while(|arg| *arg != "--")
        .collect::<Vec<_>>();
    let mut selectors = Vec::new();
    let mut limit = 1000;
    for pair in args.windows(2) {
        if pair[0] == "--trace" && selectors.len() < 64 {
            let text = pair[1];
            if !text.is_empty() && !text.starts_with("--") && text.len() <= 4096 {
                selectors.push(text.clone());
            }
        }
        if pair[0] == "--trace-limit"
            && let Ok(parsed) = pair[1].parse::<usize>()
            && (1..=10_000).contains(&parsed)
        {
            limit = parsed;
        }
    }
    report["limits"]["trace_events"] = json!(limit);
    if !selectors.is_empty()
        && report["trace"]["requested"]
            .as_array()
            .is_none_or(Vec::is_empty)
    {
        fpas_debug::batch::trace_request(report, &selectors, limit);
    }
    fpas_debug::batch::trace_coverage(report);
}
