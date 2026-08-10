//! JSONL protocol conformance at the shared debugger boundary.

#![allow(
    clippy::expect_used,
    reason = "protocol tests use expect to keep fixture failures local"
)]

use std::time::Duration;

use fpas_debug::{
    PreparedDebugTarget,
    jsonl::{JsonlServer, ServerStatus, serve_script},
};
use fpas_vm::DebugExecutionLimits;
use serde_json::{Value, json};

fn server(source: &str) -> JsonlServer {
    server_with_limits(source, DebugExecutionLimits::default())
}

fn server_with_limits(source: &str, limits: DebugExecutionLimits) -> JsonlServer {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile debug fixture");
    JsonlServer::new(PreparedDebugTarget::new(executable, Vec::new()).with_execution_limits(limits))
        .expect("create JSONL server")
}

fn request(id: u64, command: &str, arguments: Value) -> String {
    json!({"type":"request","id":id,"command":command,"arguments":arguments}).to_string()
}

#[test]
fn lifecycle_is_machine_readable_and_deterministic() {
    let script = [
        request(1, "initialize", json!({"version":2})),
        request(2, "launch", json!({"stop_on_entry":true})),
        request(3, "stack", json!({})),
        request(4, "stack", json!({})),
        request(5, "continue", json!({})),
    ]
    .join("\n");
    let mut output = Vec::new();
    serve_script(
        script.as_bytes(),
        &mut output,
        server("program Main; begin var X: integer := 1 end."),
    )
    .expect("serve script");
    let records = String::from_utf8(output).expect("UTF-8 records");
    let values = records
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("valid JSONL"))
        .collect::<Vec<_>>();
    assert!(values.iter().any(|value| value["event"] == "stopped"));
    assert!(values.iter().any(|value| value["event"] == "terminated"));
    let stacks = values
        .iter()
        .filter(|value| value["command"] == "stack")
        .collect::<Vec<_>>();
    assert_eq!(stacks.len(), 2);
    assert_eq!(stacks[0]["body"], stacks[1]["body"]);
}

#[test]
fn malformed_input_and_duplicate_ids_return_stable_errors() {
    let mut malformed = server("program Main; begin end.");
    let records = malformed.handle_line("{");
    assert_eq!(records[0]["body"]["code"], "invalid_request");
    assert_eq!(malformed.status(), ServerStatus::Terminated);

    let mut duplicate = server("program Main; begin end.");
    let _ = duplicate.handle_line(&request(1, "initialize", json!({})));
    let records = duplicate.handle_line(&request(1, "launch", json!({})));
    assert_eq!(records[0]["error"]["code"], "invalid_request");
}

#[test]
fn invalid_state_and_unsupported_commands_are_explicit() {
    let mut server = server("program Main; begin end.");
    let _ = server.handle_line(&request(1, "initialize", json!({})));
    let invalid = server.handle_line(&request(2, "continue", json!({})));
    assert_eq!(invalid[0]["error"]["code"], "invalid_state");
    let unsupported = server.handle_line(&request(3, "completions", json!({})));
    assert_eq!(unsupported[0]["error"]["code"], "unsupported_capability");
}

#[test]
fn instruction_and_timeout_limits_are_reported() {
    let cases = [
        (
            "program Main; begin while true do begin end end.",
            DebugExecutionLimits {
                max_instructions: 2,
                ..DebugExecutionLimits::default()
            },
            "instruction_limit",
        ),
        (
            "program Main; begin while true do begin end end.",
            DebugExecutionLimits {
                timeout: Duration::ZERO,
                ..DebugExecutionLimits::default()
            },
            "timeout",
        ),
    ];
    for (source, limits, code) in cases {
        let mut server = server_with_limits(source, limits);
        let _ = server.handle_line(&request(1, "initialize", json!({})));
        let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":false})));
        let records = server.wait();
        assert!(
            records.iter().any(|record| record["body"]["code"] == code),
            "missing {code}: {records:?}"
        );
    }
}

#[test]
fn initialize_reports_the_configured_execution_limits() {
    let limits = DebugExecutionLimits {
        max_instructions: 123,
        timeout: Duration::from_millis(456),
        max_output_bytes: 789,
    };
    let mut server = server_with_limits("program Main; begin end.", limits);

    let records = server.handle_line(&request(1, "initialize", json!({})));
    let reported = &records[0]["body"]["limits"];

    assert_eq!(records[0]["body"]["capabilities"]["evaluate_calls"], true);

    assert_eq!(reported["instructions"], 123);
    assert_eq!(reported["timeout_milliseconds"], 456);
    assert_eq!(reported["captured_output_bytes"], 789);
    assert_eq!(reported["evaluation_calls"], 64);
    assert_eq!(reported["evaluation_call_depth"], 32);
    assert_eq!(reported["evaluation_call_instructions"], 1_000_000);
    assert_eq!(reported["evaluation_detached_values"], 65_536);
    assert_eq!(reported["evaluation_call_timeout_milliseconds"], 2_000);
}

#[test]
fn broken_protocol_writer_is_returned_as_transport_failure() {
    struct BrokenWriter;
    impl std::io::Write for BrokenWriter {
        fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "closed",
            ))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let script = request(1, "initialize", json!({}));
    let error = serve_script(
        script.as_bytes(),
        BrokenWriter,
        server("program Main; begin end."),
    )
    .expect_err("broken writer must fail transport");
    assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
}

#[test]
fn evaluate_parses_one_read_only_expression_and_reports_stable_errors() {
    let mut server = server(
        "program Main; function Double(X: integer): integer; begin return X * 2 end; begin var X: integer := 1 end.",
    );
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = server.handle_line(&request(2, "launch", json!({"stop_on_entry":true})));

    let result = server.handle_line(&request(3, "evaluate", json!({"expression":"1 + 2 * 3"})));
    assert_eq!(result[0]["body"]["result"], "7");

    let called = server.handle_line(&request(4, "evaluate", json!({"expression":"Double(4)"})));
    assert_eq!(called[0]["body"]["result"], "8", "{called:?}");

    let unknown = server.handle_line(&request(5, "evaluate", json!({"expression":"Missing(4)"})));
    assert_eq!(unknown[0]["error"]["code"], "call_target_unknown");

    let trailing = server.handle_line(&request(6, "evaluate", json!({"expression":"1 extra"})));
    assert_eq!(trailing[0]["error"]["code"], "expression_parse");
}

#[test]
fn exact_hit_condition_stops_once_and_expires_after_the_nth_hit() {
    let mut server = loop_server();
    let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
    let set = server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":6,"hit_condition":"3"}),
    ));
    assert_eq!(set[0]["body"]["verified"], true, "{set:?}");
    let _ = server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));

    let events = wait_until_stable(&mut server);
    assert_eq!(server.status(), ServerStatus::Stopped, "{events:?}");
    let stack = server.handle_line(&request(4, "stack", json!({})));
    let frame = stack[0]["body"]["frames"][0]["frame_id"]
        .as_u64()
        .expect("frame");
    let evaluated = server.handle_line(&request(
        5,
        "evaluate",
        json!({"expression":"I","frame_id":frame}),
    ));
    assert_eq!(evaluated[0]["body"]["result"], "2");

    let _ = server.handle_line(&request(6, "continue", json!({})));
    let events = wait_until_stable(&mut server);
    assert_eq!(server.status(), ServerStatus::Terminated, "{events:?}");
}

#[test]
fn false_conditions_auto_continue_and_condition_errors_stop_closed() {
    let mut false_server = loop_server();
    let _ = false_server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = false_server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":6,"condition":"I < 0"}),
    ));
    let _ = false_server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let events = wait_until_stable(&mut false_server);
    assert_eq!(
        false_server.status(),
        ServerStatus::Terminated,
        "{events:?}"
    );
    assert!(!events.iter().any(|event| event["event"] == "stopped"));

    let mut error_server = loop_server();
    let _ = error_server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = error_server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":6,"condition":"I + 1"}),
    ));
    let _ = error_server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let events = wait_until_stable(&mut error_server);
    assert_eq!(error_server.status(), ServerStatus::Stopped, "{events:?}");
    assert!(
        events
            .iter()
            .any(|event| event["event"] == "protocol_error")
    );
}

#[test]
fn logpoints_interpolate_without_stopping_and_shared_locations_log_before_stop() {
    let mut log_server = loop_server();
    let _ = log_server.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = log_server.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":6,"log_message":"I={I}"}),
    ));
    let _ = log_server.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let events = wait_until_stable(&mut log_server);
    let output = events
        .iter()
        .filter(|event| event["event"] == "output")
        .filter_map(|event| event["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "I=0\nI=1\nI=2\nI=3\nI=4\n");
    assert_eq!(log_server.status(), ServerStatus::Terminated);

    let mut mixed = loop_server();
    let _ = mixed.handle_line(&request(1, "initialize", json!({"version":2})));
    let _ = mixed.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":6,"log_message":"before {I}"}),
    ));
    let _ = mixed.handle_line(&request(
        3,
        "breakpoint.set",
        json!({"source":"<memory>","line":6}),
    ));
    let _ = mixed.handle_line(&request(4, "launch", json!({"stop_on_entry":false})));
    let events = wait_until_stable(&mut mixed);
    let output_index = events
        .iter()
        .position(|event| event["event"] == "output")
        .expect("log output");
    let stop_index = events
        .iter()
        .position(|event| event["event"] == "stopped")
        .expect("stop event");
    assert!(output_index < stop_index, "{events:?}");
}

#[test]
fn conditions_and_logpoints_use_detached_controlled_calls() {
    let source = "program Main;\n\
                  mutable var Probe: integer := 0;\n\
                  function Matches(Value: integer): boolean;\n\
                  begin\n\
                    Probe := Probe + 1;\n\
                    return Value = 2\n\
                  end;\n\
                  function Render(Value: integer): integer;\n\
                  begin\n\
                    return Value + 10\n\
                  end;\n\
                  begin\n\
                    mutable var I: integer := 0;\n\
                    while I < 3 do\n\
                    begin\n\
                      I := I + 1\n\
                    end\n\
                  end.";
    let line = source
        .lines()
        .position(|line| line.trim() == "I := I + 1")
        .map(|line| line + 1)
        .expect("loop line");

    let mut conditional = server(source);
    let _ = conditional.handle_line(&request(1, "initialize", json!({"version":2})));
    let set = conditional.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":line,"condition":"Matches(I)"}),
    ));
    assert_eq!(set[0]["body"]["verified"], true, "{set:?}");
    let _ = conditional.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let events = wait_until_stable(&mut conditional);
    assert_eq!(conditional.status(), ServerStatus::Stopped, "{events:?}");
    let probe = conditional.handle_line(&request(4, "evaluate", json!({"expression":"Probe"})));
    assert_eq!(probe[0]["body"]["result"], "0", "{probe:?}");

    let mut logpoint = server(source);
    let _ = logpoint.handle_line(&request(1, "initialize", json!({"version":2})));
    let set = logpoint.handle_line(&request(
        2,
        "breakpoint.set",
        json!({"source":"<memory>","line":line,"log_message":"value={Render(I)}"}),
    ));
    assert_eq!(set[0]["body"]["verified"], true, "{set:?}");
    let _ = logpoint.handle_line(&request(3, "launch", json!({"stop_on_entry":false})));
    let events = wait_until_stable(&mut logpoint);
    let output = events
        .iter()
        .filter(|event| event["event"] == "output")
        .filter_map(|event| event["body"]["text"].as_str())
        .collect::<String>();
    assert_eq!(output, "value=10\nvalue=11\nvalue=12\n");
    assert_eq!(logpoint.status(), ServerStatus::Terminated);
}

#[test]
fn invalid_breakpoint_expressions_and_templates_are_unverified() {
    let cases = [
        json!({"condition":"go Work()"}),
        json!({"hit_condition":">= 3"}),
        json!({"log_message":"value={}"}),
        json!({"log_message":"value={I"}),
    ];
    for (index, fields) in cases.into_iter().enumerate() {
        let mut server = loop_server();
        let _ = server.handle_line(&request(1, "initialize", json!({"version":2})));
        let mut arguments = json!({"source":"<memory>","line":6});
        arguments
            .as_object_mut()
            .expect("arguments")
            .extend(fields.as_object().expect("fields").clone());
        let response = server.handle_line(&request(2, "breakpoint.set", arguments));
        assert_eq!(
            response[0]["body"]["verified"], false,
            "case {index}: {response:?}"
        );
    }
}

fn loop_server() -> JsonlServer {
    server(
        "program Main;\n\
         begin\n\
           mutable var I: integer := 0;\n\
           while I < 5 do\n\
           begin\n\
             I := I + 1\n\
           end\n\
         end.",
    )
}

fn wait_until_stable(server: &mut JsonlServer) -> Vec<Value> {
    let mut records = Vec::new();
    while server.status() == ServerStatus::Running {
        records.extend(server.wait());
    }
    records
}
