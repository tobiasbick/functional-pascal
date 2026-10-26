#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail directly at their observable boundary"
)]
//! Scoped MCP carrier endpoints share the SDK dispatch and request cancellation path.
#[path = "v1/peer.rs"]
#[expect(
    dead_code,
    reason = "Shared peer fixture includes helpers for other protocol contracts"
)]
mod peer;
use fpas_acp::{
    connection::{Fault, Limits, Payload, Settings},
    schema::Version,
};
use peer::{Peer, raw};

#[test]
fn nested_carriers_are_bidirectional_scoped_bounded_and_do_not_block_other_responses() {
    let mut settings = Settings::for_version(Version::V1);
    settings.experimental.mcp = true;
    let peer = Peer::configured(
        settings,
        Limits {
            routes: 2,
            ..Limits::default()
        },
    );
    peer.initialize(r#"{"protocolVersion":1,"agentCapabilities":{"loadSession":true,"mcpCapabilities":{"acp":true},"sessionCapabilities":{"close":{}}}}"#);
    let global = peer
        .connection
        .register_mcp_endpoint("global", None)
        .unwrap();
    let scoped = peer
        .connection
        .register_mcp_endpoint("scoped", Some("saved"))
        .unwrap();
    assert_eq!(
        peer.connection.register_mcp_endpoint("third", None),
        Err(Fault::LimitExceeded("Routes", 2))
    );
    assert_eq!(
        peer.connection.send(
            "mcp/message",
            raw(r#"{"serverId":"scoped","requestId":"early","method":"tools/list"}"#)
        ),
        Err(Fault::Closed)
    );
    let load = peer.connection.send("session/load", raw(r#"{"sessionId":"saved","cwd":"/workspace","mcpServers":[{"type":"acp","name":"Tools","serverId":"scoped"}]}"#)).unwrap();
    let setup = peer.take();
    // An endpoint is already routable while setup/replay has not returned.
    peer.feed(raw(r#"{"jsonrpc":"2.0","id":"scoped-in","method":"mcp/message","params":{"serverId":"scoped","requestId":"same","method":"tools/list","params":{"exact":1e400}}}"#));
    let Payload::Request(scoped_right, _, _) = peer.event().payload else {
        panic!("scoped carrier");
    };
    peer.response(&setup, "{}");
    assert!(matches!(peer.event().payload, Payload::Response(token, _, _) if token == load));
    let outgoing = peer.connection.send("mcp/message", raw(r#"{"serverId":"global","requestId":"same","method":"tools/call","params":{"n":18446744073709551615}}"#)).unwrap();
    let request = peer.take();
    peer.feed(raw(r#"{"jsonrpc":"2.0","id":"global-in","method":"mcp/message","params":{"serverId":"global","requestId":"same","method":"tools/list"}}"#));
    let Payload::Request(global_right, _, _) = peer.event().payload else {
        panic!("global carrier");
    };
    peer.exchange("_unrelated", "{}", "{}");
    peer.connection.remove_mcp_endpoint(global).unwrap();
    assert_eq!(
        peer.connection
            .respond(global_right, raw(r#"{"result":null}"#)),
        Err(Fault::Closed)
    );
    assert!(
        matches!(peer.event().payload, Payload::RequestCancelled(right) if right == global_right)
    );
    let first = peer.take();
    let second = peer.take();
    assert!([&first, &second].iter().any(|message| {
        message
            .member("method")
            .unwrap()
            .is_some_and(|method| method.text() == "\"$/cancel_request\"")
    }));
    assert!(
        [&first, &second]
            .iter()
            .any(|message| message.member("error").unwrap().is_some())
    );
    peer.response(&request, r#"{"result":null}"#);
    assert!(matches!(peer.event().payload, Payload::McpResponse(token, _, _) if token == outgoing));
    // The same inner request ID on another target still owns its independent right.
    peer.connection
        .respond(scoped_right, raw(r#"{"result":{"tools":[]}}"#))
        .unwrap();
    assert_eq!(
        peer.take().member("id").unwrap().unwrap(),
        raw(r#""scoped-in""#)
    );
    peer.connection.notify("mcp/message", raw(r#"{"serverId":"scoped","requestId":"same","method":"notifications/progress","params":{"progress":1}}"#)).unwrap();
    assert_eq!(
        peer.take().member("method").unwrap().unwrap(),
        raw(r#""mcp/message""#)
    );
    peer.exchange("session/close", r#"{"sessionId":"saved"}"#, "{}");
    assert_eq!(
        peer.connection.remove_mcp_endpoint(scoped),
        Err(Fault::Closed)
    );
    let replacement = peer
        .connection
        .register_mcp_endpoint("global", None)
        .unwrap();
    assert_ne!(global, replacement);
    assert_eq!(
        peer.connection.remove_mcp_endpoint(global),
        Err(Fault::Closed)
    );
    assert!(peer.connection.remove_mcp_endpoint(replacement).unwrap());
}
