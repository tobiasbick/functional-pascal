//! Framing and lifecycle transcripts for the FPAS DAP adapter.

#![allow(
    clippy::expect_used,
    reason = "DAP transcript fixtures use direct assertions"
)]

use std::io::{BufReader, Cursor};

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
        content: source.into(),
    }])
}

fn request(seq: u64, command: &str, arguments: Value) -> Value {
    json!({"seq":seq,"type":"request","command":command,"arguments":arguments})
}

fn jsonl_request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
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
    assert!(
        messages
            .iter()
            .any(|message| message["command"] == "evaluate" && message["success"] == false)
    );
    assert!(
        messages
            .windows(2)
            .all(|pair| pair[0]["seq"].as_u64() < pair[1]["seq"].as_u64())
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
