#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Observable MCP attachment contracts"
)]
//! Public connection setup must publish fresh authenticated MCP endpoints atomically.

#[path = "v1/peer.rs"]
#[expect(dead_code, reason = "Shared protocol fixture")]
mod peer;
use fpas_acp::connection::{Fault, Payload};
use fpas_mcp::{Catalog, HttpRuntime, Limits, Server};
use peer::{Peer, raw};
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
    time::Duration,
};

fn server() -> Server {
    Server::new(Catalog::new(Limits::default()).unwrap())
}
fn descriptor(request: &fpas_acp::RawJson) -> serde_json::Value {
    let value: serde_json::Value = serde_json::from_str(request.text()).unwrap();
    value["params"]["mcpServers"][0].clone()
}
fn post(descriptor: &serde_json::Value) -> std::io::Result<String> {
    let address = descriptor["url"]
        .as_str()
        .unwrap()
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap();
    let mut stream = TcpStream::connect_timeout(&address.parse().unwrap(), Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
    write!(
        stream,
        "POST /mcp HTTP/1.1\r\nHost: {address}\r\nAuthorization: {}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2025-06-18\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        descriptor["headers"][0]["value"].as_str().unwrap(),
        body.len()
    )?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply)?;
    Ok(reply)
}
#[test]
fn unsupported_http_capability_preserves_borrowed_server_ownership() {
    let peer = Peer::new();
    peer.initialize(r#"{"protocolVersion":1,"agentCapabilities":{}}"#);
    let runtime = Arc::new(HttpRuntime::new().unwrap());
    let server = server();
    peer.connection
        .attach_tool_server("application", server.clone(), runtime, false)
        .unwrap();
    assert!(matches!(
        peer.connection.send(
            "session/new",
            raw(r#"{"cwd":"/workspace","mcpServers":[]}"#)
        ),
        Err(Fault::UnsupportedCapability("McpHttp"))
    ));
    assert!(server.endpoint().is_ok());
    peer.connection.close().unwrap();
    assert!(matches!(peer.event().payload, Payload::Closed(_)));
    assert!(!server.is_closed());
}
#[test]
fn fresh_session_access_is_retired_on_close_and_owned_server_closes_with_connection() {
    let peer = Peer::new();
    peer.initialize(r#"{"protocolVersion":1,"agentCapabilities":{"mcpCapabilities":{"http":true},"sessionCapabilities":{"close":{}}}}"#);
    let runtime = Arc::new(HttpRuntime::new().unwrap());
    let borrowed = server();
    peer.connection
        .attach_tool_server("application", borrowed.clone(), runtime.clone(), false)
        .unwrap();
    assert!(matches!(peer.connection.send("session/new", raw(r#"{"cwd":"/workspace","mcpServers":[{"type":"http","name":"application","url":"http://127.0.0.1:1/mcp","headers":[]}]}"#)), Err(Fault::InvalidArgument("McpServerName"))));
    peer.connection
        .send(
            "session/new",
            raw(r#"{"cwd":"/workspace","mcpServers":[]}"#),
        )
        .unwrap();
    let first = peer.take();
    let access = descriptor(&first);
    assert!(post(&access).unwrap().contains("200 OK"));
    peer.response(&first, r#"{"sessionId":"first"}"#);
    assert!(matches!(peer.event().payload, Payload::Response(..)));
    peer.connection
        .send(
            "session/new",
            raw(r#"{"cwd":"/workspace","mcpServers":[]}"#),
        )
        .unwrap();
    let second = peer.take();
    let replacement = descriptor(&second);
    assert!(
        access["headers"] != replacement["headers"],
        "replacement credentials must be fresh"
    );
    assert_ne!(access["url"], replacement["url"]);
    peer.response(&second, r#"{"sessionId":"second"}"#);
    peer.event();
    peer.connection
        .send("session/close", raw(r#"{"sessionId":"first"}"#))
        .unwrap();
    let closing = peer.take();
    assert!(post(&access).is_err());
    assert!(post(&replacement).unwrap().contains("200 OK"));
    peer.response(&closing, "{}");
    assert!(matches!(peer.event().payload, Payload::Response(..)));
    let (first, receipts) = peer.connection.detach_tool_server("application").unwrap();
    assert!(first);
    for receipt in receipts {
        peer::wait(receipt.wait()).unwrap();
    }
    assert!(!peer.connection.detach_tool_server("application").unwrap().0);
    assert!(post(&replacement).is_err());
    let owned = server();
    peer.connection
        .attach_tool_server("owned", owned.clone(), runtime, true)
        .unwrap();
    peer.connection.close().unwrap();
    assert!(owned.is_closed());
    assert!(matches!(peer.event().payload, Payload::Closed(_)));
    assert!(post(&replacement).is_err());
    assert!(!borrowed.is_closed());
    assert!(borrowed.endpoint().is_ok());
}

#[test]
fn failed_setup_retires_access_and_returns_endpoint_capacity() {
    let peer = Peer::new();
    peer.initialize(
        r#"{"protocolVersion":1,"agentCapabilities":{"mcpCapabilities":{"http":true}}}"#,
    );
    let runtime = Arc::new(HttpRuntime::new().unwrap());
    let borrowed = Server::new(
        Catalog::new(Limits {
            endpoints: 1,
            ..Limits::default()
        })
        .unwrap(),
    );
    peer.connection
        .attach_tool_server("application", borrowed.clone(), runtime, false)
        .unwrap();
    peer.connection
        .send(
            "session/new",
            raw(r#"{"cwd":"/workspace","mcpServers":[]}"#),
        )
        .unwrap();
    let request = peer.take();
    let access = descriptor(&request);
    let id = request.member("id").unwrap().unwrap();
    peer.feed(raw(&format!(
        r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32603,"message":"Setup rejected"}}}}"#,
        id.text()
    )));
    assert!(matches!(peer.event().payload, Payload::RequestFailed(..)));
    let (_, receipts) = peer.connection.detach_tool_server("application").unwrap();
    for receipt in receipts {
        peer::wait(receipt.wait()).unwrap();
    }
    assert!(post(&access).is_err());
    assert!(borrowed.endpoint().is_ok());
    peer.connection.close().unwrap();
    assert!(matches!(peer.event().payload, Payload::Closed(_)));
}
