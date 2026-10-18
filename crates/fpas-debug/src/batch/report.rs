//! Versioned batch document and its final escaped-byte ceiling.

use serde_json::{Value, json};

/// Maximum complete JSON document, including its newline.
pub const MAX_REPORT_BYTES: usize = 8 * 1024 * 1024;

/// Builds a complete empty envelope for errors before a target can be prepared.
pub fn empty_report(outcome: &str, exit_code: i32) -> Value {
    json!({"schema_version":1,"outcome":outcome,"exit_code":exit_code,"stop":null,"diagnostics":[],"sources":[],"globals":[],"tasks":[],"expressions":[],"output":[],"capture":{"incomplete":false,"reasons":[],"total_tasks":null,"total_globals":null},"limits":{"timeout_seconds":30,"instruction_limit":100000000,"output_bytes":1048576,"report_bytes":MAX_REPORT_BYTES,"tasks":64,"frames_per_task":32,"frames":256,"bindings":4096,"depth":8,"children":64,"string_scalars":4096}})
}

/// Creates one structured request, execution or transport diagnostic.
pub fn diagnostic(code: &str, stage: &str, message: &str) -> Value {
    json!({"code":code,"stage":stage,"message":message.chars().take(4096).collect::<String>(),"truncated":message.chars().nth(4096).is_some()})
}

/// Serializes a complete bounded report, preserving outcome and explicit omitted-data markers.
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
                break;
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
        // Keep the primary diagnostic/outcome even if many bounded diagnostics fill control space.
        if let Some(records) = report["diagnostics"].as_array_mut() {
            records.truncate(1);
        }
        report["capture"]["incomplete"] = json!(true);
        report["capture"]["reasons"] = json!(["byte_limit", "diagnostic_limit"]);
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

fn encoded_size(value: &Value) -> usize {
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
