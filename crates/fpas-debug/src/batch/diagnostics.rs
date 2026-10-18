//! Bounded structured diagnostics that preserve the primary FP code and outcome.

use fpas_diagnostics::Diagnostic;
use serde_json::{Value, json};

fn prefix(text: &str) -> String {
    text.chars().take(4096).collect()
}

/// Renders a coded diagnostic without retaining unbounded panic or compiler text.
pub fn fp_diagnostic(path: Option<&str>, source: Option<&str>, original: &Diagnostic) -> Value {
    let mut clipped = false;
    let mut text = |value: &str| {
        let bounded = prefix(value);
        clipped |= bounded.len() < value.len();
        bounded
    };
    let mut bounded = Diagnostic::error_without_source(
        original.code,
        text(&original.message),
        original.help.as_deref().map(&mut text),
    );
    bounded.severity = original.severity;
    bounded.span = original.span;
    bounded.expected = original.expected.as_deref().map(&mut text).map(Into::into);
    bounded.found = original.found.as_deref().map(&mut text).map(Into::into);
    let path = path.map(&mut text);
    // Keep only complete fix-its: clipping replacement text would change their meaning.
    for fix in original.fixes.iter().take(4) {
        if fix.title.len() <= 4096
            && fix.edits.len() <= 8
            && fix.edits.iter().all(|edit| edit.new_text.len() <= 4096)
        {
            bounded.fixes.push(fix.clone());
        } else {
            clipped = true;
        }
    }
    clipped |= bounded.fixes.len() < original.fixes.len();
    let mut record: Value = serde_json::from_str(
        &fpas_diagnostics::render_json(path.as_deref(), source, &bounded)
            .expect("structured diagnostic"),
    )
    .expect("diagnostic JSON");
    record["truncated"] = json!(clipped);
    record
}
