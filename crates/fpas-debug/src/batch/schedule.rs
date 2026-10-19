//! Scheduling coverage survives worker failures and final report clipping.

use serde_json::{Value, json};

/// Mark coverage without inventing driver counters or changing the primary exit.
pub fn coverage(report: &mut Value) {
    if report["schedule"]["mode"] != "seeded" {
        return;
    }
    let has = |code: &str| {
        report["diagnostics"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item["code"] == code))
    };
    let reason = if has("execution_timeout") || has("timeout") {
        Some(("interrupted", "host_deadline"))
    } else if has("external_cancel") {
        Some(("interrupted", "external_cancel"))
    } else if has("worker_failure") || has("worker_start") {
        Some(("unavailable", "worker_failure"))
    } else if report["capture"]["reasons"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item == "state_unavailable"))
        && report["exit_code"] == 3
    {
        Some(("unavailable", "state_unavailable"))
    } else {
        None
    };
    if let Some((status, reason)) = reason {
        report["schedule"]["status"] = json!(status);
        super::trace::append_reason(&mut report["schedule"]["reasons"], json!(reason));
    }
}
