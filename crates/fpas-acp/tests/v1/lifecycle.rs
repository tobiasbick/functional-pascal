use super::peer::{Peer, raw};
use fpas_acp::{
    connection::{Fault, Limits, Payload, Settings},
    schema::Version,
};

#[test]
fn close_invalidates_all_session_requests_before_reopening_the_same_id() {
    let mut settings = Settings::for_version(Version::V1);
    settings.client_capabilities = fpas_acp::codec::decode(raw(
        r#"{"fs":{"writeTextFile":true},"terminal":true,"elicitation":{"form":{}}}"#,
    ))
    .unwrap()
    .value;
    let peer = Peer::configured(settings, Limits::default());
    peer.initialize(r#"{"protocolVersion":1,"agentCapabilities":{"loadSession":true,"sessionCapabilities":{"close":{}}}}"#);
    peer.session("s");
    peer.session("other");
    let requests = [
        (
            "fs/write_text_file",
            r#"{"sessionId":"s","path":"/workspace/file.txt","content":"old"}"#,
        ),
        ("terminal/create", r#"{"sessionId":"s","command":"unused"}"#),
        (
            "elicitation/create",
            r#"{"sessionId":"s","mode":"form","message":"old","requestedSchema":{"type":"object","properties":{}}}"#,
        ),
        (
            "fs/write_text_file",
            r#"{"sessionId":"other","path":"/workspace/other.txt","content":"other"}"#,
        ),
    ];
    let mut responders = Vec::new();
    for (id, (method, params)) in requests.iter().enumerate() {
        peer.feed(raw(&format!(
            r#"{{"jsonrpc":"2.0","id":{id},"method":"{method}","params":{params}}}"#
        )));
        let Payload::Request(responder, _, _) = peer.event().payload else {
            panic!("host request")
        };
        responders.push(responder);
    }
    let close = peer
        .connection
        .send("session/close", raw(r#"{"sessionId":"s"}"#))
        .unwrap();
    let request = peer.take();
    let id = request.member("id").unwrap().unwrap();
    peer.feed(raw(&format!(
        r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32603,"message":"retry"}}}}"#,
        id.text()
    )));
    assert!(matches!(peer.event().payload, Payload::RequestFailed(token, _) if token == close));
    assert!(
        peer.connection
            .incoming(responders[0], requests[0].0)
            .is_ok()
    );

    peer.exchange("session/close", r#"{"sessionId":"s"}"#, "{}");
    for responder in &responders[..3] {
        assert!(
            matches!(peer.event().payload, Payload::RequestCancelled(actual) if actual == *responder)
        );
        assert!(peer.take().member("error").unwrap().is_some());
    }
    peer.exchange(
        "session/load",
        r#"{"sessionId":"s","cwd":"/workspace","mcpServers":[]}"#,
        "{}",
    );
    for index in 0..2 {
        assert!(matches!(
            peer.connection.claim_host(
                responders[index],
                requests[index].0,
                &raw(requests[index].1)
            ),
            Err(Fault::Closed)
        ));
        assert_eq!(
            peer.connection.respond(responders[index], raw("{}")),
            Err(Fault::Closed)
        );
    }
    assert_eq!(
        peer.connection
            .validate_interaction(responders[2], &raw(r#"{"action":"cancel"}"#)),
        Err(Fault::Closed)
    );
    let claim = peer
        .connection
        .claim_host(responders[3], requests[3].0, &raw(requests[3].1))
        .unwrap();
    drop(claim);
    peer.connection.respond(responders[3], raw("{}")).unwrap();
    peer.take();
}

#[test]
fn close_releases_nested_observed_sessions_and_their_mcp_routes() {
    let mut settings = Settings::for_version(Version::V1);
    settings.experimental.subagents = true;
    settings.experimental.mcp = true;
    settings.client_capabilities = fpas_acp::codec::decode(raw(r#"{"subagents":{}}"#))
        .unwrap()
        .value;
    let peer = Peer::configured(
        settings,
        Limits {
            sessions: 4,
            ..Limits::default()
        },
    );
    peer.initialize(r#"{"protocolVersion":1,"agentCapabilities":{"mcpCapabilities":{"acp":true},"sessionCapabilities":{"close":{}}}}"#);
    peer.session("other");
    let other = peer
        .connection
        .register_mcp_endpoint("other-mcp", Some("other"))
        .unwrap();
    for _ in 0..3 {
        peer.session("parent");
        for (parent, child) in [("parent", "child"), ("child", "grandchild")] {
            peer.update(parent, &format!(r#"{{"sessionUpdate":"subagent_update","sessionId":"{child}","capabilities":{{"cancel":{{}}}}}}"#));
            assert!(matches!(peer.event().payload, Payload::Notification(_, _)));
        }
        let child = peer
            .connection
            .register_mcp_endpoint("child-mcp", Some("grandchild"))
            .unwrap();
        peer.exchange("session/close", r#"{"sessionId":"parent"}"#, "{}");
        assert_eq!(peer.connection.mcp_endpoint(child), Err(Fault::Closed));
        assert!(peer.connection.mcp_endpoint(other).is_ok());
        assert_eq!(
            peer.connection
                .notify("session/cancel", raw(r#"{"sessionId":"grandchild"}"#)),
            Err(Fault::InvalidArgument("SessionId"))
        );
    }
    peer.exchange(
        "session/prompt",
        r#"{"sessionId":"other","prompt":[]}"#,
        r#"{"stopReason":"end_turn"}"#,
    );
}
