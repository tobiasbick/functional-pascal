//! Versioned batch document and its final escaped-byte ceiling.

use serde_json::{Value, json};

/// Maximum complete JSON document, including its newline.
pub const MAX_REPORT_BYTES: usize = 8 * 1024 * 1024;

/// Builds a complete empty envelope for errors before a target can be prepared.
pub fn empty_report(outcome: &str, exit_code: i32) -> Value {
    let mut report = json!({"schema_version":1,"outcome":outcome,"exit_code":exit_code,"stop":null,"diagnostics":[],"sources":[],"globals":[],"tasks":[],"expressions":[],"output":[],"trace":{"requested":[],"watches":[],"events":[],"complete":true,"reasons":[],"event_count":0,"total_count":0},"capture":{"incomplete":false,"reasons":[],"total_tasks":null,"total_globals":null},"limits":{"timeout_seconds":30,"instruction_limit":100000000,"output_bytes":1048576,"report_bytes":MAX_REPORT_BYTES,"tasks":64,"frames_per_task":32,"frames":256,"bindings":4096,"depth":8,"children":64,"string_scalars":4096,"trace_events":1000,"trace_watches":64,"trace_field_depth":16,"trace_path_depth":16,"trace_value_bytes":65536,"trace_retained_value_bytes":8388608,"trace_live_instances":1024,"trace_local_retained_value_bytes":8388608,"trace_work_bytes":16777216,"trace_bytes":2097152}});
    report["schedule"] = json!(fpas_vm::DebugScheduleReport::requested(None));
    report
}

/// Creates one structured request, execution or transport diagnostic.
pub fn diagnostic(code: &str, stage: &str, message: &str) -> Value {
    json!({"code":code,"stage":stage,"message":message.chars().take(4096).collect::<String>(),"truncated":message.chars().nth(4096).is_some()})
}

/// Serializes a complete bounded report, preserving outcome and explicit omitted-data markers.
#[expect(
    clippy::unwrap_used,
    reason = "Batch envelopes originate from empty_report, which initializes the fixed array fields."
)]
pub fn finish_report(report: &mut Value) -> Vec<u8> {
    if report["diagnostics"]
        .as_array()
        .is_some_and(|records| records.iter().any(|record| record["truncated"] == true))
    {
        report["capture"]["incomplete"] = json!(true);
        if !report["capture"]["reasons"]
            .as_array()
            .is_some_and(|r| r.iter().any(|r| r == "byte_limit"))
        {
            report["capture"]["reasons"]
                .as_array_mut()
                .unwrap()
                .push(json!("byte_limit"));
        }
    }

    let mut trace_bytes = encoded_size(&report["trace"]["events"]);
    let mut remaining = encoded_size(report).saturating_sub(MAX_REPORT_BYTES - 16_384);
    if remaining > 0 {
        report["capture"]["incomplete"] = json!(true);
        if !report["capture"]["reasons"]
            .as_array()
            .is_some_and(|r| r.iter().any(|r| r == "byte_limit"))
        {
            report["capture"]["reasons"]
                .as_array_mut()
                .unwrap()
                .push(json!("byte_limit"));
        }
        while remaining > 0 {
            let removed = shrink_snapshot(report);
            if removed == 0 {
                break;
            }
            remaining = remaining.saturating_sub(removed);
        }
        while remaining > 0 {
            let removed = if let Some(record) = report["output"].as_array_mut().and_then(Vec::pop) {
                if let Some(last) = report["output"]
                    .as_array_mut()
                    .and_then(|records| records.last_mut())
                {
                    last["truncated"] = json!(true);
                }
                encoded_size(&record)
            } else if let Some(source) = report["sources"].as_array_mut().and_then(Vec::pop) {
                encoded_size(&source)
            } else {
                let removed = shrink_requested(report, remaining, &mut trace_bytes);
                if removed == 0 {
                    break;
                }
                removed
            };
            remaining = remaining.saturating_sub(removed);
        }
    }
    loop {
        let mut bytes = serde_json::to_vec(report).unwrap_or_default();
        if bytes.len() < MAX_REPORT_BYTES {
            bytes.push(b'\n');
            return bytes;
        }
        report["capture"]["incomplete"] = json!(true);
        if !report["capture"]["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| reason == "byte_limit"))
        {
            report["capture"]["reasons"]
                .as_array_mut()
                .unwrap()
                .push(json!("byte_limit"));
        }
        if let Some(output) = report["output"]
            .as_array_mut()
            .and_then(|output| output.last_mut())
        {
            if let Some(text) = output["text"].as_str() {
                let maximum = text.chars().count() / 2;
                if maximum > 0 {
                    output["text"] = json!(text.chars().take(maximum).collect::<String>());
                    output["truncated"] = json!(true);
                    continue;
                }
            }
            report["output"].as_array_mut().unwrap().pop();
            continue;
        }
        if report["sources"]
            .as_array_mut()
            .is_some_and(|sources| sources.pop().is_some())
        {
            continue;
        }
        if shrink_snapshot(report) > 0
            || shrink_requested(
                report,
                bytes.len().saturating_sub(MAX_REPORT_BYTES - 1),
                &mut trace_bytes,
            ) > 0
        {
            continue;
        }
        // Keep the primary diagnostic/outcome even if many bounded diagnostics fill control space.
        if let Some(records) = report["diagnostics"].as_array_mut() {
            records.truncate(1);
        }
        report["capture"]["incomplete"] = json!(true);
        super::trace::append_reason(&mut report["capture"]["reasons"], json!("diagnostic_limit"));
        // Valid requests have bounded control fields. This final defensive path also
        // guarantees progress for a malformed worker document without looping forever.
        if let Some(expressions) = report["expressions"].as_array_mut()
            && expressions.pop().is_some()
        {
            continue;
        }
        let removed_watch = report["trace"]["watches"]
            .as_array_mut()
            .is_some_and(|v| v.pop().is_some());
        let removed_request = report["trace"]["requested"]
            .as_array_mut()
            .is_some_and(|v| v.pop().is_some());
        if removed_watch || removed_request {
            report["trace"]["complete"] = json!(false);
            super::trace::append_reason(&mut report["trace"]["reasons"], json!("byte_limit"));
            super::trace::coverage(report);
            continue;
        }
        // Valid control data cannot reach this path. A malformed worker envelope
        // must still terminate and preserve the primary diagnostic and outcome.
        let old = std::mem::replace(report, empty_report("error", 3));
        if encoded_size(&old["schedule"]) <= 1024 {
            report["schedule"] = old["schedule"].clone();
        } else {
            let seed = old["schedule"]["seed"]
                .as_str()
                .and_then(|text| text.parse::<u64>().ok());
            report["schedule"] = json!(fpas_vm::DebugScheduleReport::requested(seed));
            if seed.is_some() {
                report["schedule"]["status"] = json!("unavailable");
                report["schedule"]["reasons"] = json!(["state_unavailable"]);
            }
        }
        report["outcome"] = old["outcome"].clone();
        report["exit_code"] = if old["exit_code"] == 0 {
            json!(3)
        } else {
            old["exit_code"].clone()
        };
        let primary = &old["diagnostics"][0];
        if let Some(message) = primary["message"].as_str() {
            report["diagnostics"] = json!([diagnostic(
                &primary["code"]
                    .as_str()
                    .unwrap_or("worker_failure")
                    .chars()
                    .take(64)
                    .collect::<String>(),
                &primary["stage"]
                    .as_str()
                    .unwrap_or("capture")
                    .chars()
                    .take(64)
                    .collect::<String>(),
                message,
            )]);
        }
        report["capture"]["incomplete"] = json!(true);
        report["capture"]["reasons"] = json!(["byte_limit", "state_unavailable"]);
        let selectors = old["trace"]["requested"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .take(64)
            .map(|text| text.chars().take(4096).collect::<String>())
            .collect::<Vec<_>>();
        super::trace::request(report, &selectors, 1000);
        if !selectors.is_empty() {
            report["trace"]["reasons"] = json!(["byte_limit", "state_unavailable"]);
        }
    }
}

fn shrink_snapshot(report: &mut Value) -> usize {
    if let Some(global) = report["globals"].as_array_mut().and_then(Vec::pop) {
        return encoded_size(&global);
    }

    if let Some(tasks) = report["tasks"].as_array_mut() {
        for task in tasks.iter_mut().rev() {
            if let Some(frames) = task["frames"].as_array_mut() {
                for frame in frames.iter_mut().rev() {
                    if let Some(binding) = frame["bindings"].as_array_mut().and_then(Vec::pop) {
                        frame["truncated"] = json!(true);
                        return encoded_size(&binding);
                    }
                }
                if let Some(frame) = frames.pop() {
                    task["truncated"] = json!(true);
                    return encoded_size(&frame);
                }
            }
        }
        if let Some(task) = tasks.pop() {
            return encoded_size(&task);
        }
    }
    0
}

fn shrink_requested(report: &mut Value, required: usize, trace_bytes: &mut usize) -> usize {
    // If all trace events could not make enough room, a requested value must be
    // clipped anyway. Keep its companion history rather than discarding it first.
    if required > *trace_bytes {
        let removed = shrink_expression(report);
        if removed > 0 {
            return removed;
        }
    }
    let removed = super::trace::shrink(report);
    if removed > 0 {
        *trace_bytes = trace_bytes.saturating_sub(removed + 1);
        removed
    } else {
        shrink_expression(report)
    }
}

fn shrink_expression(report: &mut Value) -> usize {
    if let Some(expressions) = report["expressions"].as_array_mut() {
        for expression in expressions.iter_mut().rev() {
            if !expression["value"].is_null() {
                let removed = encoded_size(&expression["value"]);
                expression["value"] = Value::Null;
                expression["error"] = diagnostic(
                    "value_limit",
                    "capture",
                    "Requested value did not fit the report.",
                );
                if report["exit_code"] == 0 {
                    report["exit_code"] = json!(3);
                }
                return removed.saturating_sub(256).max(1);
            }
        }
    }
    0
}

pub(super) fn encoded_size(value: &Value) -> usize {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.saturating_add(bytes.len());
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    let _ = serde_json::to_writer(&mut counter, value);
    counter.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_trace_loses_only_tail_events_and_preserves_runtime_failure() {
        let mut report = empty_report("runtime_error", 2);
        report["diagnostics"] = json!([diagnostic("FP5010", "execution", "primary panic")]);
        super::super::trace::request(&mut report, &["Total".into()], 1000);
        report["trace"]["complete"] = json!(true);
        report["trace"]["reasons"] = json!([]);
        report["trace"]["events"] = json!(
            (1..=1000)
                .map(|sequence| {
                    json!({"sequence":sequence,"new_value":{"scalar":"\u{1}".repeat(2000)}})
                })
                .collect::<Vec<_>>()
        );
        report["trace"]["event_count"] = json!(1000);
        report["trace"]["total_count"] = json!(1000);
        let bytes = finish_report(&mut report);
        assert!(bytes.len() <= MAX_REPORT_BYTES);
        assert_eq!(report["exit_code"], 2);
        assert_eq!(report["diagnostics"][0]["code"], "FP5010");
        assert_eq!(report["trace"]["complete"], false);
        assert_eq!(report["trace"]["total_count"], 1000);
        assert_eq!(report["capture"]["incomplete"], true);
        let events = report["trace"]["events"].as_array().unwrap();
        assert!(events.len() < 1000);
        for (i, event) in events.iter().enumerate() {
            assert_eq!(event["sequence"], i + 1);
        }
        assert_eq!(report["trace"]["event_count"], events.len());
    }

    #[test]
    fn automatic_snapshot_is_removed_before_requested_trace_and_print() {
        let mut report = empty_report("completed", 0);
        report["globals"] = json!([{"value":"\u{1}".repeat(1_400_000)}]);
        report["trace"]["events"] = json!([{"sequence":1,"new_value":{"scalar":"42"}}]);
        report["trace"]["event_count"] = json!(1);
        report["trace"]["total_count"] = json!(1);
        report["expressions"] = json!([{"expression":"Total","value":{"scalar":"42"}}]);
        finish_report(&mut report);
        assert!(report["globals"].as_array().unwrap().is_empty());
        assert_eq!(report["trace"]["event_count"], 1);
        assert_eq!(report["trace"]["complete"], true);
        assert_eq!(report["expressions"][0]["value"]["scalar"], "42");
        assert_eq!(report["exit_code"], 0);
    }

    #[test]
    fn snapshot_clipping_preserves_bounded_scheduling_identity_and_counters() {
        let mut report = empty_report("runtime_error", 2);
        report["schedule"] = json!(fpas_vm::DebugScheduleReport::requested(Some(u64::MAX)));
        report["schedule"]["status"] = json!("controlled");
        report["schedule"]["dispatches"] = json!("42");
        let schedule = report["schedule"].clone();
        report["globals"] = json!([{"value":"\u{1}".repeat(1_400_000)}]);
        let bytes = finish_report(&mut report);
        assert!(bytes.len() <= MAX_REPORT_BYTES);
        assert_eq!(report["schedule"], schedule);
        assert_eq!(report["exit_code"], 2);
    }

    #[test]
    fn malformed_oversized_schedule_cannot_make_report_reduction_loop() {
        let mut report = empty_report("error", 3);
        report["schedule"] = json!(fpas_vm::DebugScheduleReport::requested(Some(42)));
        report["schedule"]["unexpected"] = json!("x".repeat(MAX_REPORT_BYTES));
        let bytes = finish_report(&mut report);
        assert!(bytes.len() <= MAX_REPORT_BYTES);
        assert_eq!(report["schedule"]["seed"], "42");
        assert_eq!(report["schedule"]["status"], "unavailable");
        assert_eq!(report["schedule"]["dispatches"], Value::Null);
    }
}
