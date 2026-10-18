//! The bounded, versioned source-query wire contract and snapshot-bound locations.

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

pub(super) fn empty(args: &[String], cwd: &Path) -> Value {
    let operation = args
        .get(1)
        .map(String::as_str)
        .filter(|s| ["refs", "definition", "type"].contains(s));
    let mut tokens = args.iter().skip(2);
    let mut position = None;
    while let Some(token) = tokens.next() {
        if ["--context", "--std-lib", "--timeout", "--limit"].contains(&token.as_str()) {
            tokens.next();
        } else if !token.starts_with('-') {
            position = Some(token);
            break;
        }
    }
    let mut components = position.map(|s| s.rsplitn(3, ':'));
    let column = components
        .as_mut()
        .and_then(Iterator::next)
        .and_then(|s| s.parse::<usize>().ok());
    let line = components
        .as_mut()
        .and_then(Iterator::next)
        .and_then(|s| s.parse::<usize>().ok());
    let file = components
        .as_mut()
        .and_then(Iterator::next)
        .map(|s| path(&cwd.join(s), cwd));
    json!({
        "schema_version": 1, "operation": operation, "outcome": "error", "exit_code": 1,
        "request": {"file":file,"line":line,"column":column},
        "context": {"kind":null,"manifest":null,"projects":[],"path_base":cwd.to_string_lossy().replace('\\',"/")},
        "target":null,"matches":[],"type":null,"diagnostics":[],
        "coverage":{"complete":false,"reasons":["state_unavailable"],"files":[],"returned_count":0,"total_count":null},
        "limits":{"timeout_seconds":30,"result_count":if operation == Some("refs") {Some(1000)} else {None},
            "source_files":4096,"source_file_bytes":8388608,"source_total_bytes":67108864,
            "json_bytes":MAX_JSON,"text_scalars":4096,"preview_scalars":160}
    })
}

pub(super) fn diagnostic(report: &mut Value, code: &str, message: &str) {
    if let Some(items) = report["diagnostics"].as_array_mut() {
        items.push(json!({"kind":"query","code":code,"severity":"error","message":message}));
    }
}

pub(super) fn incomplete(report: &mut Value, reason: &str) {
    report["outcome"] = json!("incomplete");
    report["exit_code"] = json!(3);
    report["coverage"]["complete"] = json!(false);
    report["coverage"]["total_count"] = Value::Null;
    if let Some(items) = report["coverage"]["reasons"].as_array_mut()
        && !items.contains(&json!(reason))
    {
        items.push(json!(reason));
    }
}

pub(super) fn failure(args: &[String], cwd: &Path, code: &str, message: &str, exit: i32) -> Value {
    let mut report = empty(args, cwd);
    report["exit_code"] = json!(exit);
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

pub(super) fn location(snapshot: &DocumentSnapshot, span: SourceSpan, cwd: &Path) -> Option<Value> {
    let position = |offset| {
        let pos = snapshot.line_index().position(offset)?;
        let start = snapshot.line_index().line_start(pos.line)?;
        Some(
            json!({"line":pos.line+1,"column":snapshot.source().get(start..offset)?.chars().count()+1}),
        )
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
    Some(
        json!({"path":path(snapshot.path(),cwd),"range":{"start":start,"end":end},
        "byte_span":{"start":span.offset(),"length":span.length()},
        "preview":{"text":preview,"column_start":skip+1,"truncated":skip>0 || text.chars().count()>skip+160}}),
    )
}

fn clip_text(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            let mut clipped = Vec::new();
            for (name, value) in fields.iter_mut() {
                if [
                    "display", "message", "expected", "found", "hint", "title", "new_text",
                ]
                .contains(&name.as_str())
                    && let Some(text) = value.as_str()
                    && text.chars().count() > 4096
                {
                    *value = json!(text.chars().take(4096).collect::<String>());
                    clipped.push(format!("{name}_truncated"));
                } else {
                    clip_text(value);
                }
            }
            for name in clipped {
                fields.insert(name, json!(true));
            }
        }
        Value::Array(values) => {
            for value in values {
                clip_text(value);
            }
        }
        _ => {}
    }
}

pub(super) fn finish(report: &mut Value) -> Vec<u8> {
    clip_text(report);
    loop {
        let mut bytes = serde_json::to_vec(report).unwrap_or_default();
        bytes.push(b'\n');
        if bytes.len() <= MAX_JSON {
            return bytes;
        }
        incomplete(report, "json_limit");
        // Keep the first useful match and primary diagnostic while shedding bulk.
        let mut removed = false;
        for key in ["matches", "diagnostics"] {
            if let Some(items) = report[key].as_array_mut()
                && items.len() > 1
            {
                items.truncate(items.len().div_ceil(2));
                removed = true;
                break;
            }
        }
        if !removed
            && let Some(items) = report["coverage"]["files"].as_array_mut()
            && !items.is_empty()
        {
            items.truncate(items.len() / 2);
            removed = true;
        }
        if !removed {
            if let Some(first) = report["diagnostics"]
                .as_array_mut()
                .and_then(|d| d.first_mut())
            {
                first["fixes"] = json!([]);
                first["details_omitted"] = json!(true);
            }
            // Required identities can themselves be adversarially long.
            for key in ["target", "type"] {
                report[key] = Value::Null;
            }
            report["context"]["projects"] = json!([]);
        }
        report["coverage"]["returned_count"] =
            json!(report["matches"].as_array().map_or(0, Vec::len));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escaped_budget_keeps_primary_diagnostic_and_marks_lost_matches() {
        let mut report = empty(
            &["query".into(), "refs".into(), "file.fpas:1:1".into()],
            Path::new("/workspace"),
        );
        report["outcome"] = json!("ok");
        report["exit_code"] = json!(0);
        report["coverage"]["complete"] = json!(true);
        diagnostic(&mut report, "FP3003", &"\u{1}".repeat(9000));
        let entry = json!({"path":"a".repeat(1024),"byte_span":{"start":0,"length":1},"preview":{"text":"\u{1}".repeat(160),"truncated":true}});
        report["matches"] = json!(vec![entry; 10000]);
        let bytes = finish(&mut report);
        assert!(bytes.len() <= MAX_JSON);
        assert_eq!(report["diagnostics"][0]["code"], "FP3003");
        assert_eq!(report["diagnostics"][0]["message_truncated"], true);
        assert_eq!(report["coverage"]["complete"], false);
        assert_eq!(report["exit_code"], 3);
        assert!(report["coverage"]["total_count"].is_null());
        assert!(!report["matches"].as_array().is_none_or(Vec::is_empty));
        assert!(serde_json::from_slice::<Value>(&bytes).is_ok());
    }
}
