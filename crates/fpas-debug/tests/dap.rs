//! Framing and lifecycle transcripts for the FPAS DAP adapter.

#![allow(
    clippy::expect_used,
    reason = "DAP transcript fixtures use direct assertions"
)]

mod support;

use std::io::{BufReader, Cursor};
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use fpas_debug::{
    DebugSourceContent, PreparedDebugTarget,
    dap::{DapServer, read_message, serve, write_message},
    jsonl::JsonlServer,
};
use serde_json::{Value, json};

fn server(source: &str) -> DapServer {
    DapServer::new(target(source)).expect("create DAP server")
}

fn target(source: &str) -> PreparedDebugTarget {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty());
    let executable = fpas_compiler::compile(&program).expect("compile DAP fixture");
    PreparedDebugTarget::new(executable, Vec::new()).with_sources(vec![DebugSourceContent {
        path: "<memory>".into(),
        original_path: None,
        content: source.into(),
    }])
}

fn request(seq: u64, command: &str, arguments: Value) -> Value {
    json!({"seq":seq,"type":"request","command":command,"arguments":arguments})
}

fn jsonl_request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

fn handle_and_wait(adapter: &mut DapServer, request: Value) -> Vec<Value> {
    let mut messages = adapter.handle(request);
    if messages.is_empty() {
        messages.extend(adapter.wait());
    }
    messages
}

#[test]
fn framing_uses_utf8_byte_lengths_and_rejects_truncation() {
    let message = json!({"text":"Grüße"});
    let mut framed = Vec::new();
    write_message(&mut framed, &message).expect("write framed message");
    let decoded = read_message(&mut BufReader::new(framed.as_slice())).expect("read frame");
    assert_eq!(decoded, Some(message));

    let mut truncated = BufReader::new(Cursor::new(b"Content-Length: 5\r\n\r\n{}"));
    let error = read_message(&mut truncated).expect_err("truncated body rejected");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

fn target_with_sources(source: &str, sources: Vec<DebugSourceContent>) -> PreparedDebugTarget {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty());
    let executable = fpas_compiler::compile(&program).expect("compile DAP fixture");
    PreparedDebugTarget::new(executable, Vec::new()).with_sources(sources)
}

#[test]
fn framing_rejects_oversized_or_excessive_headers() {
    let oversized = format!("X-Debug: {}\r\n", "x".repeat(8 * 1024));
    let error = read_message(&mut BufReader::new(oversized.as_bytes()))
        .expect_err("oversized header line must be rejected");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("8 KiB"), "{error}");

    let mut excessive = String::from("Content-Length: 2\r\n");
    for _ in 0..64 {
        excessive.push_str("X-Debug: value\r\n");
    }
    excessive.push_str("\r\n{}");
    let error = read_message(&mut BufReader::new(excessive.as_bytes()))
        .expect_err("excessive header count must be rejected");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("count"), "{error}");

    let mut excessive_bytes = String::from("Content-Length: 2\r\n");
    for _ in 0..63 {
        excessive_bytes.push_str("X-Debug: ");
        excessive_bytes.push_str(&"x".repeat(1_040));
        excessive_bytes.push_str("\r\n");
    }
    excessive_bytes.push_str("\r\n{}");
    let error = read_message(&mut BufReader::new(excessive_bytes.as_bytes()))
        .expect_err("excessive total header bytes must be rejected");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("64 KiB"), "{error}");
}

#[test]
fn framing_accepts_maximum_body_size() {
    const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

    let mut input = format!("Content-Length: {MAX_BODY_BYTES}\r\n\r\n").into_bytes();
    input.extend_from_slice(b"{}");
    input.resize(input.len() + MAX_BODY_BYTES - 2, b' ');

    let message = read_message(&mut BufReader::new(input.as_slice()))
        .expect("maximum body must parse")
        .expect("message");
    assert_eq!(message, json!({}));
}

#[test]
fn supported_lifecycle_and_unsupported_request_are_explicit() {
    let requests = [
        request(1, "initialize", json!({})),
        request(2, "launch", json!({"stopOnEntry":true})),
        request(
            3,
            "setBreakpoints",
            json!({"source":{"path":"C:/workspace/<memory>"},"breakpoints":[{"line":1}]}),
        ),
        request(4, "configurationDone", json!({})),
        request(5, "threads", json!({})),
        request(6, "evaluate", json!({"expression":"1"})),
        request(7, "disconnect", json!({})),
    ];
    let mut input = Vec::new();
    for request in requests {
        write_message(&mut input, &request).expect("frame request");
    }
    let mut output = Vec::new();
    serve(
        Cursor::new(input),
        &mut output,
        server("program Main; begin var X: integer := 1 end."),
    )
    .expect("serve DAP transcript");
    let mut reader = BufReader::new(output.as_slice());
    let mut messages = Vec::new();
    while let Some(message) = read_message(&mut reader).expect("read response") {
        messages.push(message);
    }
    assert!(
        messages
            .iter()
            .any(|message| message["event"] == "initialized")
    );
    assert!(messages.iter().any(|message| {
        message["command"] == "initialize" && message["body"]["supportsVariablePaging"] == true
    }));
    assert!(messages.iter().any(|message| message["event"] == "stopped"));
    assert!(
        messages.iter().any(|message| {
            message["command"] == "setBreakpoints"
                && message["body"]["breakpoints"][0]["verified"] == true
        }),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message["command"] == "threads" && message["success"] == true)
    );
    assert!(messages.iter().any(|message| {
        message["command"] == "evaluate"
            && message["success"] == true
            && message["body"]["result"] == "1"
    }));
    assert!(messages.iter().any(|message| {
        message["command"] == "initialize"
            && message["body"]["supportsEvaluateForHovers"] == true
            && message["body"]["supportsConditionalBreakpoints"] == true
            && message["body"]["supportsHitConditionalBreakpoints"] == true
            && message["body"]["supportsLogPoints"] == true
    }));
    assert!(
        messages
            .windows(2)
            .all(|pair| pair[0]["seq"].as_u64() < pair[1]["seq"].as_u64())
    );
}

#[test]
fn disconnect_drops_an_input_that_stays_open() {
    let mut input = Vec::new();
    write_message(&mut input, &request(1, "initialize", json!({}))).expect("frame initialize");
    write_message(&mut input, &request(2, "disconnect", json!({}))).expect("frame disconnect");
    let (reader, dropped, _keep_open) = support::open_reader(input);

    serve(reader, Vec::new(), server("program Main; begin end."))
        .expect("disconnect open DAP input");

    assert!(
        dropped.load(Ordering::Acquire),
        "DAP reader remained owned by a detached thread"
    );
}

#[test]
fn inspection_source_and_step_requests_use_the_shared_session() {
    let source = "program Main; begin mutable var X: integer := 1; X := X + 1 end.";
    let mut adapter = server(source);
    let initialized = adapter.handle(request(1, "initialize", json!({})));
    assert!(
        initialized
            .iter()
            .any(|message| message["event"] == "initialized")
    );
    assert!(
        adapter.handle(request(2, "launch", json!({"stopOnEntry":true})))[0]["success"] == true
    );
    let configured = adapter.handle(request(3, "configurationDone", json!({})));
    assert!(
        configured
            .iter()
            .any(|message| message["event"] == "stopped")
    );

    let stack = adapter.handle(request(4, "stackTrace", json!({"threadId":1})));
    let frame_id = stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("frame ID");
    let scopes = adapter.handle(request(5, "scopes", json!({"frameId":frame_id})));
    let variables_reference = scopes[0]["body"]["scopes"]
        .as_array()
        .expect("scopes")
        .iter()
        .find_map(|scope| {
            scope["variablesReference"]
                .as_u64()
                .filter(|reference| *reference != 0)
        })
        .expect("inspectable scope");
    let variables = adapter.handle(request(
        6,
        "variables",
        json!({"variablesReference":variables_reference}),
    ));
    assert!(variables[0]["success"] == true);

    let source_response = adapter.handle(request(
        7,
        "source",
        json!({"source":{"path":"C:/workspace/<memory>"},"sourceReference":0}),
    ));
    assert_eq!(source_response[0]["body"]["content"], source);

    let step = adapter.handle(request(8, "stepIn", json!({"threadId":1})));
    assert!(step[0]["success"] == true);
    let events = adapter.wait();
    assert!(
        events
            .iter()
            .any(|message| matches!(message["event"].as_str(), Some("stopped" | "terminated")))
    );
}

#[test]
fn jsonl_and_dap_report_equivalent_entry_stack() {
    let source = "program Main; begin var X: integer := 1 end.";
    let mut jsonl = JsonlServer::new(target(source)).expect("JSONL server");
    let _ = jsonl.handle_line(&jsonl_request(1, "initialize", json!({})));
    let _ = jsonl.handle_line(&jsonl_request(2, "launch", json!({"stop_on_entry":true})));
    let jsonl_stack = jsonl.handle_line(&jsonl_request(3, "stack", json!({})));

    let mut dap = server(source);
    let _ = dap.handle(request(1, "initialize", json!({})));
    let _ = dap.handle(request(2, "launch", json!({"stopOnEntry":true})));
    let _ = dap.handle(request(3, "configurationDone", json!({})));
    let dap_stack = dap.handle(request(4, "stackTrace", json!({"threadId":1})));

    assert_eq!(
        jsonl_stack[0]["body"]["frames"][0]["name"],
        dap_stack[0]["body"]["stackFrames"][0]["name"]
    );
    assert_eq!(
        jsonl_stack[0]["body"]["frames"][0]["location"]["source"],
        dap_stack[0]["body"]["stackFrames"][0]["source"]["path"]
    );
    assert_eq!(
        jsonl_stack[0]["body"]["frames"][0]["location"]["line"],
        dap_stack[0]["body"]["stackFrames"][0]["line"]
    );

    let jsonl_frame = jsonl_stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("JSONL frame");
    let jsonl_scopes =
        jsonl.handle_line(&jsonl_request(4, "scopes", json!({"frame_id":jsonl_frame})));
    let jsonl_reference =
        first_reference(&jsonl_scopes[0]["body"]["scopes"], "variables_reference");
    let jsonl_variables = jsonl.handle_line(&jsonl_request(
        5,
        "variables",
        json!({"variables_reference":jsonl_reference}),
    ));

    let dap_frame = dap_stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("DAP frame");
    let dap_scopes = dap.handle(request(5, "scopes", json!({"frameId":dap_frame})));
    let dap_reference = first_reference(&dap_scopes[0]["body"]["scopes"], "variablesReference");
    let dap_variables = dap.handle(request(
        6,
        "variables",
        json!({"variablesReference":dap_reference}),
    ));
    assert_eq!(
        variable_pairs(&jsonl_variables[0]["body"]["variables"]),
        variable_pairs(&dap_variables[0]["body"]["variables"])
    );
}

fn first_reference(scopes: &Value, field: &str) -> u64 {
    scopes
        .as_array()
        .expect("scopes")
        .iter()
        .find_map(|scope| scope[field].as_u64().filter(|reference| *reference != 0))
        .expect("variables reference")
}

fn variable_pairs(variables: &Value) -> Vec<(String, String)> {
    variables
        .as_array()
        .expect("variables")
        .iter()
        .map(|variable| {
            (
                variable["name"].as_str().unwrap_or_default().to_string(),
                variable["value"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

#[test]
fn every_step_request_is_accepted_from_a_stable_stop() {
    for command in ["stepIn", "next", "stepOut"] {
        let mut adapter = server("program Main; begin var X: integer := 1 end.");
        let _ = adapter.handle(request(1, "initialize", json!({})));
        let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":true})));
        let _ = adapter.handle(request(3, "configurationDone", json!({})));
        let response = adapter.handle(request(4, command, json!({"threadId":1})));
        assert_eq!(response[0]["success"], true, "{command}: {response:?}");
        let events = adapter.wait();
        assert!(
            events
                .iter()
                .any(|message| matches!(message["event"].as_str(), Some("stopped" | "terminated"))),
            "{command}: {events:?}"
        );
    }
}

#[test]
fn live_pause_and_disconnect_cancel_owned_execution() {
    let requests = [
        request(1, "initialize", json!({})),
        request(2, "launch", json!({"stopOnEntry":false})),
        request(3, "configurationDone", json!({})),
        request(4, "pause", json!({"threadId":1})),
        request(5, "disconnect", json!({"terminateDebuggee":true})),
    ];
    let mut input = Vec::new();
    for request in requests {
        write_message(&mut input, &request).expect("frame request");
    }
    let mut output = Vec::new();
    serve(
        Cursor::new(input),
        &mut output,
        server("program Main; begin while true do begin end end."),
    )
    .expect("serve pause transcript");
    let mut reader = BufReader::new(output.as_slice());
    let mut messages = Vec::new();
    while let Some(message) = read_message(&mut reader).expect("read response") {
        messages.push(message);
    }
    assert!(
        messages
            .iter()
            .any(|message| message["command"] == "pause" && message["success"] == true)
    );
    assert!(
        messages
            .iter()
            .any(|message| message["event"] == "stopped" && message["body"]["reason"] == "pause")
    );
    assert!(
        messages
            .iter()
            .any(|message| message["event"] == "terminated")
    );
}

#[test]
fn runtime_failure_is_inspectable_then_continue_terminates() {
    let mut adapter =
        server("program Main; begin var Zero: integer := 0; var Value: integer := 1 div Zero end.");
    let _ = adapter.handle(request(1, "initialize", json!({})));
    let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":false})));
    let _ = adapter.handle(request(3, "configurationDone", json!({})));
    let failed = adapter.wait();
    assert!(
        failed.iter().any(
            |message| message["event"] == "stopped" && message["body"]["reason"] == "exception"
        )
    );
    let stack = adapter.handle(request(4, "stackTrace", json!({"threadId":1})));
    assert_eq!(stack[0]["success"], true);
    let continued = adapter.handle(request(5, "continue", json!({"threadId":1})));
    assert!(
        continued
            .iter()
            .any(|message| message["command"] == "continue" && message["success"] == true)
    );
    assert!(
        continued
            .iter()
            .any(|message| message["event"] == "terminated")
    );
}

#[test]
fn evaluate_contexts_share_frame_results_and_controlled_calls() {
    let mut adapter = server(
        "program Main; function Double(X: integer): integer; begin return X * 2 end; begin var X: integer := 1 end.",
    );
    let initialized = adapter.handle(request(1, "initialize", json!({})));
    assert_eq!(initialized[0]["body"]["supportsEvaluateForHovers"], true);
    let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":true})));
    let _ = adapter.handle(request(3, "configurationDone", json!({})));
    let stack = adapter.handle(request(4, "stackTrace", json!({"threadId":1})));
    let frame = stack[0]["body"]["stackFrames"][0]["id"]
        .as_u64()
        .expect("frame");

    for (index, context) in ["watch", "repl", "hover", "variables"]
        .into_iter()
        .enumerate()
    {
        let response = handle_and_wait(
            &mut adapter,
            request(
                10 + index as u64,
                "evaluate",
                json!({"expression":"1 + 2", "frameId":frame, "context":context}),
            ),
        );
        assert_eq!(response[0]["success"], true, "{context}: {response:?}");
        assert_eq!(response[0]["body"]["result"], "3");
    }

    let called = handle_and_wait(
        &mut adapter,
        request(
            20,
            "evaluate",
            json!({"expression":"Double(4)", "frameId":frame, "context":"watch"}),
        ),
    );
    assert_eq!(called[0]["success"], true, "{called:?}");
    assert_eq!(called[0]["body"]["result"], "8");

    let globals_only = handle_and_wait(
        &mut adapter,
        request(21, "evaluate", json!({"expression":"X", "context":"watch"})),
    );
    assert_eq!(globals_only[0]["success"], false);

    let evaluating = adapter.handle(request(
        22,
        "evaluate",
        json!({"expression":"Double(5)", "frameId":frame, "context":"watch"}),
    ));
    assert!(evaluating.is_empty());
    let continued = adapter.handle(request(23, "continue", json!({"threadId":1})));
    assert!(continued.iter().any(|message| {
        message["command"] == "evaluate"
            && message["success"] == true
            && message["body"]["result"] == "10"
    }));
    assert!(
        continued
            .iter()
            .any(|message| { message["command"] == "continue" && message["success"] == true })
    );
}

#[test]
fn cancel_and_disconnect_reach_active_call_evaluation() {
    let source = "program Main; function Loop(X: integer): integer; begin mutable var I: integer := X; while I < 1000000000 do I := I + 1; return I end; begin var X: integer := 1 end.";
    let mut adapter = server(source);
    let initialized = adapter.handle(request(1, "initialize", json!({})));
    assert_eq!(initialized[0]["body"]["supportsCancelRequest"], true);
    let _ = adapter.handle(request(2, "launch", json!({"stopOnEntry":true})));
    let _ = adapter.handle(request(3, "configurationDone", json!({})));

    let evaluating = adapter.handle(request(
        4,
        "evaluate",
        json!({"expression":"Loop(0)", "context":"watch"}),
    ));
    assert!(evaluating.is_empty(), "call should run asynchronously");
    let mut cancelled = adapter.handle(request(5, "cancel", json!({"requestId":4})));
    if !cancelled
        .iter()
        .any(|message| message["command"] == "evaluate")
    {
        cancelled.extend(adapter.wait());
    }
    assert!(
        cancelled
            .iter()
            .any(|message| { message["command"] == "cancel" && message["success"] == true })
    );
    assert!(cancelled.iter().any(|message| {
        message["command"] == "evaluate"
            && message["success"] == false
            && message["message"]
                .as_str()
                .is_some_and(|message| message.contains("cancelled"))
    }));
    assert_eq!(
        adapter.handle(request(6, "stackTrace", json!({"threadId":1})))[0]["success"],
        true
    );

    let evaluating = adapter.handle(request(
        7,
        "evaluate",
        json!({"expression":"Loop(0)", "context":"watch"}),
    ));
    assert!(evaluating.is_empty());
    let disconnected = adapter.handle(request(8, "disconnect", json!({})));
    assert!(
        disconnected
            .iter()
            .any(|message| { message["command"] == "evaluate" && message["success"] == false })
    );
    assert!(
        disconnected
            .iter()
            .any(|message| { message["command"] == "disconnect" && message["success"] == true })
    );
    assert!(
        disconnected
            .iter()
            .any(|message| message["event"] == "terminated")
    );
}

#[test]
fn dap_hit_conditions_and_logpoints_match_jsonl_policy() {
    let source = "program Main;\n\
                  begin\n\
                    mutable var I: integer := 0;\n\
                    while I < 5 do\n\
                    begin\n\
                      I := I + 1\n\
                    end\n\
                  end.";
    let mut hit_adapter = server(source);
    let _ = hit_adapter.handle(request(1, "initialize", json!({})));
    let _ = hit_adapter.handle(request(2, "launch", json!({})));
    let breakpoints = hit_adapter.handle(request(
        3,
        "setBreakpoints",
        json!({
            "source":{"path":"<memory>"},
            "breakpoints":[{"line":6,"condition":"I >= 0","hitCondition":"3"}]
        }),
    ));
    assert_eq!(breakpoints[0]["body"]["breakpoints"][0]["verified"], true);
    let _ = hit_adapter.handle(request(4, "configurationDone", json!({})));
    let events = wait_dap_until_stable(&mut hit_adapter);
    assert!(events.iter().any(|message| {
        message["event"] == "stopped" && message["body"]["reason"] == "breakpoint"
    }));

    let mut log_adapter = server(source);
    let _ = log_adapter.handle(request(1, "initialize", json!({})));
    let _ = log_adapter.handle(request(2, "launch", json!({})));
    let _ = log_adapter.handle(request(
        3,
        "setBreakpoints",
        json!({
            "source":{"path":"<memory>"},
            "breakpoints":[{"line":6,"logMessage":"I={I}"}]
        }),
    ));
    let _ = log_adapter.handle(request(4, "configurationDone", json!({})));
    let events = wait_dap_until_stable(&mut log_adapter);
    let output = events
        .iter()
        .filter(|message| message["event"] == "output")
        .filter_map(|message| message["body"]["output"].as_str())
        .collect::<String>();
    assert_eq!(output, "I=0\nI=1\nI=2\nI=3\nI=4\n");
    assert!(!events.iter().any(|message| message["event"] == "stopped"));
    assert!(
        events
            .iter()
            .any(|message| message["event"] == "terminated")
    );
}

#[test]
fn mixed_source_breakpoints_report_each_location_and_clear_without_leaks() {
    let source = "program Main;\nbegin\n  mutable var X: integer := 1;\n  X := X + 1\nend.";
    let mut adapter = server(source);
    let _ = adapter.handle(request(1, "initialize", json!({})));
    let _ = adapter.handle(request(2, "launch", json!({})));
    let replaced = adapter.handle(request(
        3,
        "setBreakpoints",
        json!({
            "source":{"path":"<memory>"},
            "breakpoints":[{"line":4},{"line":0}]
        }),
    ));
    assert_eq!(replaced[0]["success"], true, "{replaced:?}");
    assert_eq!(
        replaced[0]["body"]["breakpoints"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(replaced[0]["body"]["breakpoints"][0]["verified"], true);
    assert_eq!(replaced[0]["body"]["breakpoints"][1]["verified"], false);

    let cleared = adapter.handle(request(
        4,
        "setBreakpoints",
        json!({"source":{"path":"<memory>"},"breakpoints":[]}),
    ));
    assert_eq!(cleared[0]["success"], true, "{cleared:?}");
    let _ = adapter.handle(request(5, "configurationDone", json!({})));
    let events = wait_dap_until_stable(&mut adapter);
    assert!(!events.iter().any(|message| message["event"] == "stopped"));
    assert!(
        events
            .iter()
            .any(|message| message["event"] == "terminated")
    );
}

#[test]
fn source_lookup_uses_original_alias_without_exposing_it() {
    let portable = "sources/0/main.fpas";
    let original = "C:\\Workspace\\Outside\\Main.fpas";
    let source = "program Main; begin end.";
    let mut adapter = DapServer::new(target_with_sources(
        source,
        vec![DebugSourceContent {
            path: portable.to_string(),
            original_path: Some(PathBuf::from(original)),
            content: source.to_string(),
        }],
    ))
    .expect("create aliased DAP server");
    let _ = adapter.handle(request(1, "initialize", json!({})));
    let response = adapter.handle(request(
        2,
        "source",
        json!({"source":{"path":"c:/workspace/outside/main.fpas"}}),
    ));
    assert_eq!(response[0]["success"], true, "{response:?}");
    assert_eq!(response[0]["body"]["content"], source);
    assert!(!response[0].to_string().contains(original));
}

#[test]
fn breakpoint_lookup_uses_windows_original_alias_and_returns_portable_path() {
    let source = "program Main;\nbegin\n  var X: integer := 1\nend.";
    let mut adapter = DapServer::new(target_with_sources(
        source,
        vec![DebugSourceContent {
            path: "<memory>".to_string(),
            original_path: Some(PathBuf::from("C:\\Workspace\\Main.fpas")),
            content: source.to_string(),
        }],
    ))
    .expect("create aliased DAP server");
    let _ = adapter.handle(request(1, "initialize", json!({})));
    let _ = adapter.handle(request(2, "launch", json!({})));
    let response = adapter.handle(request(
        3,
        "setBreakpoints",
        json!({
            "source":{"path":"c:/workspace/MAIN.fpas"},
            "breakpoints":[{"line":3}]
        }),
    ));
    assert_eq!(response[0]["success"], true, "{response:?}");
    assert_eq!(response[0]["body"]["breakpoints"][0]["verified"], true);
    assert_eq!(
        response[0]["body"]["breakpoints"][0]["source"]["path"],
        "<memory>"
    );
}

#[test]
fn ambiguous_source_suffix_does_not_bind() {
    let source = "program Main; begin end.";
    let mut adapter = DapServer::new(target_with_sources(
        source,
        vec![
            DebugSourceContent {
                path: "left/main.fpas".to_string(),
                original_path: None,
                content: "left".to_string(),
            },
            DebugSourceContent {
                path: "right/main.fpas".to_string(),
                original_path: None,
                content: "right".to_string(),
            },
        ],
    ))
    .expect("create ambiguous DAP server");
    let _ = adapter.handle(request(1, "initialize", json!({})));
    let response = adapter.handle(request(
        2,
        "source",
        json!({"source":{"path":"workspace/main.fpas"}}),
    ));
    assert_eq!(response[0]["success"], false, "{response:?}");
    assert!(
        response[0]["message"]
            .as_str()
            .is_some_and(|message| { message.to_ascii_lowercase().contains("ambiguous") })
    );
}

fn wait_dap_until_stable(adapter: &mut DapServer) -> Vec<Value> {
    let mut messages = Vec::new();
    while adapter.is_running() {
        messages.extend(adapter.wait());
    }
    messages
}
