//! Real LSP completion coverage for recovered recursive native receiver types.
//! Reference: `docs/pascal/tools/editor-integration.md`.

use super::*;

#[test]
fn nested_result_completion_and_buffer_edits_use_exact_utf16_ranges() {
    let temp = TempDirectory::new("native-result-completion");
    let source = "program Demo;\nbegin\n  const Value: Result of Result of string, integer, boolean := Ok(Ok('x'));\n  discard Value.Unwrap().Unwrap().\nend.\n";
    let edited = "program Demo;\nbegin\n  const Value: Result of Result of string, integer, boolean := Ok(Ok('x'));\n  const Music: string := '𝄞'; discard Value.Unwrap().Unwrap().LeTail;\nend.\n";
    temp.write("main.fpas", source);
    let uri = temp.uri("main.fpas");
    let dot = source.find("Unwrap().\n").expect("dot") + "Unwrap().".len();
    let prefix = edited.find("LeTail").expect("identifier");
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, None)),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&uri, source)),
        TranscriptStep::Message(completion(2, &uri, position(source, dot))),
        TranscriptStep::Message(json!({
            "jsonrpc": "2.0", "method": "textDocument/didChange",
            "params": {"textDocument": {"uri": uri, "version": 2}, "contentChanges": [{"text": edited}]}
        })),
        TranscriptStep::Message(completion(3, &uri, position(edited, prefix + 2))),
        TranscriptStep::Message(shutdown(4)),
        TranscriptStep::Message(exit()),
    ]);
    assert_success(&transcript.output);
    let full = response(&transcript.messages, 2);
    assert_string_catalog(full);
    for label in ["Length", "Slice", "IsEmpty"] {
        let candidate = item(full, label);
        assert_eq!(
            candidate["textEdit"]["range"]["start"],
            position(source, dot)
        );
        assert_eq!(candidate["textEdit"]["range"]["end"], position(source, dot));
    }
    let filtered = response(&transcript.messages, 3);
    assert_eq!(filtered["result"].as_array().expect("items").len(), 1);
    let length = item(filtered, "Length");
    assert_eq!(length["detail"], "string.Length(): integer");
    assert_eq!(length["textEdit"]["newText"], "Length");
    assert_eq!(
        length["textEdit"]["range"]["start"],
        position(edited, prefix)
    );
    assert_eq!(
        length["textEdit"]["range"]["end"],
        position(edited, prefix + "LeTail".len())
    );
}

#[test]
fn nested_result_callback_completion_uses_the_string_catalog() {
    let temp = TempDirectory::new("native-result-callback");
    let source = "program Demo;\nbegin\n  const Items: array of integer := [1];\n  discard Items.Map(F := function(X: integer): Result of Result of string, integer, boolean begin return Ok(Ok('x')); end function)[0].Unwrap().Unwrap().\nend.\n";
    temp.write("main.fpas", source);
    let uri = temp.uri("main.fpas");
    let cursor = source.find("Unwrap().\n").expect("dot") + "Unwrap().".len();
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, None)),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&uri, source)),
        TranscriptStep::Message(completion(2, &uri, position(source, cursor))),
        TranscriptStep::Message(shutdown(3)),
        TranscriptStep::Message(exit()),
    ]);
    assert_success(&transcript.output);
    assert_string_catalog(response(&transcript.messages, 2));
}

fn assert_string_catalog(response: &Value) {
    let candidates = response["result"].as_array().expect("completion array");
    assert_eq!(candidates.len(), 31, "{response}");
    assert!(
        candidates.iter().all(|candidate| {
            candidate["detail"]
                .as_str()
                .is_some_and(|detail| detail.starts_with("string."))
                && candidate["documentation"]["value"]
                    .as_str()
                    .is_some_and(|text| !text.is_empty())
                && candidate["additionalTextEdits"].is_null()
        }),
        "{response}"
    );
    for label in ["Length", "Slice", "IsEmpty"] {
        item(response, label);
    }
}

fn assert_success(output: &std::process::Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
