//! DAP global assignment attached to source and data breakpoints.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "protocol tests keep fixture failures local"
)]

use fpas_debug::{PreparedDebugTarget, dap::DapServer};
use serde_json::{Value, json};

const SOURCE: &str = r#"program BreakpointAssign;

mutable var Flag: integer := 0;

begin
  Flag := 1;
  Flag := 2
end.
"#;

fn server() -> DapServer {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile assign fixture");
    DapServer::new(PreparedDebugTarget::new(executable, Vec::new())).expect("DAP server")
}

fn send(server: &mut DapServer, seq: &mut u64, command: &str, arguments: Value) -> Vec<Value> {
    *seq += 1;
    server.handle(json!({
        "seq":*seq,"type":"request","command":command,"arguments":arguments
    }))
}

fn start(server: &mut DapServer, seq: &mut u64) {
    let _ = send(server, seq, "initialize", json!({}));
    let _ = send(server, seq, "launch", json!({"stopOnEntry":true}));
    let configured = send(server, seq, "configurationDone", json!({}));
    assert!(
        configured
            .iter()
            .any(|message| message["event"] == "stopped"),
        "{configured:?}"
    );
}

fn continue_until_stopped(server: &mut DapServer, seq: &mut u64) -> Vec<Value> {
    let mut stopped = send(server, seq, "continue", json!({"threadId":1}));
    if !stopped.iter().any(|message| message["event"] == "stopped") {
        stopped = server.wait();
    }
    stopped
}

fn line(needle: &str) -> usize {
    SOURCE
        .lines()
        .position(|line| line.contains(needle))
        .expect("marker")
        + 1
}

fn evaluate_flag(server: &mut DapServer, seq: &mut u64) -> String {
    let mut records = send(server, seq, "evaluate", json!({"expression":"Flag"}));
    if records.is_empty() {
        records = server.wait();
    }
    records[0]["body"]["result"]
        .as_str()
        .unwrap_or_else(|| panic!("Flag: {records:?}"))
        .to_string()
}

#[test]
fn dap_source_breakpoint_assign_commits_before_the_line() {
    let mut server = server();
    let mut seq = 0;
    start(&mut server, &mut seq);

    let set = send(
        &mut server,
        &mut seq,
        "setBreakpoints",
        json!({
            "source":{"path":"<memory>"},
            "breakpoints":[{
                "line":line("Flag := 2"),
                "assign":{"identity":{"index":0},"expression":"99"}
            }]
        }),
    );
    assert_eq!(set[0]["success"], true, "{set:?}");
    assert_eq!(set[0]["body"]["breakpoints"][0]["verified"], true);

    let stopped = continue_until_stopped(&mut server, &mut seq);
    assert!(
        stopped.iter().any(|message| message["event"] == "stopped"),
        "{stopped:?}"
    );
    assert_eq!(evaluate_flag(&mut server, &mut seq), "99");
}

#[test]
fn dap_frame_assign_is_rejected_without_creating_the_breakpoint() {
    let mut server = server();
    let mut seq = 0;
    start(&mut server, &mut seq);
    let rejected = send(
        &mut server,
        &mut seq,
        "setBreakpoints",
        json!({
            "source":{"path":"<memory>"},
            "breakpoints":[{
                "line":line("Flag := 2"),
                "assign":{
                    "identity":{"task_id":0,"function":0,"register":0},
                    "expression":"99"
                }
            }]
        }),
    );
    assert_eq!(rejected[0]["success"], true, "{rejected:?}");
    assert_eq!(
        rejected[0]["body"]["breakpoints"][0]["verified"], false,
        "{rejected:?}"
    );
    assert_eq!(rejected.len(), 1, "{rejected:?}");

    let continued = send(&mut server, &mut seq, "continue", json!({"threadId":1}));
    let mut records = continued;
    while !records
        .iter()
        .any(|message| message["event"] == "terminated")
    {
        let next = server.wait();
        if next.is_empty() {
            break;
        }
        records.extend(next);
    }
    assert!(
        records
            .iter()
            .any(|message| message["event"] == "terminated"),
        "{records:?}"
    );
}

#[test]
fn dap_failed_assign_stops_without_mutating() {
    let mut server = server();
    let mut seq = 0;
    start(&mut server, &mut seq);
    let set = send(
        &mut server,
        &mut seq,
        "setBreakpoints",
        json!({
            "source":{"path":"<memory>"},
            "breakpoints":[{
                "line":line("Flag := 2"),
                "assign":{"identity":{"index":0},"expression":"true"}
            }]
        }),
    );
    assert_eq!(set[0]["success"], true, "{set:?}");

    let stopped = continue_until_stopped(&mut server, &mut seq);
    assert!(
        stopped.iter().any(|message| message["event"] == "stopped"),
        "{stopped:?}"
    );
    assert_eq!(evaluate_flag(&mut server, &mut seq), "1");
}

#[test]
fn dap_data_breakpoint_assign_runs_after_the_watched_store() {
    let mut server = server();
    let mut seq = 0;
    start(&mut server, &mut seq);
    let set = send(
        &mut server,
        &mut seq,
        "setDataBreakpoints",
        json!({
            "breakpoints":[{
                "dataId":"g:0",
                "accessType":"write",
                "assign":{"identity":{"index":0},"expression":"0"}
            }]
        }),
    );
    assert_eq!(set[0]["success"], true, "{set:?}");
    assert_eq!(set[0]["body"]["breakpoints"][0]["verified"], true);

    let stopped = continue_until_stopped(&mut server, &mut seq);
    let event = stopped
        .iter()
        .find(|message| message["event"] == "stopped")
        .expect("data stop");
    assert_eq!(event["body"]["reason"], "data breakpoint", "{stopped:?}");
    assert_eq!(evaluate_flag(&mut server, &mut seq), "0");
}
