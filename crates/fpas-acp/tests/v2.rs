#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail directly at their observable boundary"
)]
//! V2 wire contracts use distinct capabilities, auth and asynchronous prompt state.
#[path = "v1/peer.rs"]
#[expect(
    dead_code,
    reason = "Shared peer methods also support stable v1 regressions"
)]
mod peer;
use fpas_acp::{
    connection::{Fault, Lifecycle, Payload, Settings},
    protocol::v1,
    schema::Version,
};
use peer::{Peer, raw};

fn configured(lower: bool) -> Peer {
    let mut settings = Settings::for_version(Version::V2);
    if lower {
        settings.supported_versions.push(Version::V1);
    }
    settings.client_info = Some(v1::Implementation::new("test", "1"));
    Peer::configured(settings, Default::default())
}
fn init(peer: &Peer, result: &str, selected: Version) {
    let token = peer
        .connection
        .send("initialize", peer.offer.clone())
        .unwrap();
    peer.response(&peer.take(), result);
    assert!(
        matches!(peer.event().payload, Payload::Initialized(actual, family, _) if actual == token && family == selected)
    );
    assert_eq!(peer.connection.version(), selected);
    assert_eq!(peer.connection.state(), Ok(Lifecycle::Ready));
}
fn exchange(peer: &Peer, method: &str, params: &str, response: &str) {
    let token = peer.connection.send(method, raw(params)).unwrap();
    let request = peer.take();
    assert_eq!(request.member("params").unwrap().unwrap(), raw(params));
    peer.response(&request, response);
    assert!(
        matches!(peer.event().payload, Payload::Response(actual, Version::V2, _) if actual == token)
    );
}
#[test]
fn v2_acceptance_and_idle_are_distinct_and_baseline_methods_need_no_v1_capabilities() {
    let peer = configured(false);
    init(
        &peer,
        r#"{"protocolVersion":2,"info":{"name":"peer","version":"1"},"capabilities":{"session":{"delete":{}}},"authMethods":[{"type":"agent","methodId":"token","name":"Token"}]}"#,
        Version::V2,
    );
    exchange(&peer, "auth/login", r#"{"methodId":"token"}"#, "{}");
    exchange(&peer, "auth/logout", "{}", "{}");
    exchange(
        &peer,
        "session/new",
        r#"{"cwd":"/workspace","mcpServers":[]}"#,
        r#"{"sessionId":"s","configOptions":[{"configId":"model","name":"Model","type":"select","currentValue":"m","options":[{"value":"m","name":"Model"}]}]}"#,
    );
    exchange(
        &peer,
        "session/set_config_option",
        r#"{"sessionId":"s","configId":"model","type":"id","value":"m"}"#,
        r#"{"configOptions":[]}"#,
    );
    exchange(
        &peer,
        "session/list",
        r#"{"cursor":"page"}"#,
        r#"{"sessions":[],"nextCursor":"next"}"#,
    );
    let first = peer
        .connection
        .send(
            "session/prompt",
            raw(r#"{"sessionId":"s","prompt":[{"type":"text","text":"one"}]}"#),
        )
        .unwrap();
    let request = peer.take();
    peer.update("s", r#"{"sessionUpdate":"state_update","state":"running"}"#);
    assert!(matches!(
        peer.event().payload,
        Payload::Notification(Version::V2, _)
    ));
    peer.response(&request, r#"{"messageId":"first"}"#);
    assert!(
        matches!(peer.event().payload, Payload::Response(token, Version::V2, _) if token == first)
    );
    // Acceptance leaves the session available while foreground work still runs.
    exchange(
        &peer,
        "session/prompt",
        r#"{"sessionId":"s","prompt":[]}"#,
        r#"{"messageId":"second"}"#,
    );
    peer.update(
        "s",
        r#"{"sessionUpdate":"state_update","state":"idle","stopReason":"end_turn"}"#,
    );
    assert!(matches!(
        peer.event().payload,
        Payload::Notification(Version::V2, _)
    ));
    let token = peer
        .connection
        .send(
            "session/resume",
            raw(r#"{"sessionId":"saved","cwd":"/workspace"}"#),
        )
        .unwrap();
    let request = peer.take();
    let replay = peer.update("saved", r#"{"sessionUpdate":"agent_message_chunk","messageId":"m","content":{"type":"text","text":"replay"}}"#);
    peer.response(&request, "{}");
    assert_eq!(peer.event().raw, Some(replay));
    assert!(
        matches!(peer.event().payload, Payload::Response(actual, Version::V2, _) if actual == token)
    );
    exchange(&peer, "session/close", r#"{"sessionId":"saved"}"#, "{}");
    exchange(&peer, "session/delete", r#"{"sessionId":"saved"}"#, "{}");
    assert!(
        peer.connection
            .send(
                "session/load",
                raw(r#"{"sessionId":"s","cwd":"/x","mcpServers":[]}"#)
            )
            .is_err()
    );
    assert!(peer.reader.try_receive().unwrap().is_none());
}
#[test]
fn lower_version_outcome_changes_the_explicit_family_without_resending_initialization() {
    let peer = configured(true);
    let initialize = peer
        .connection
        .send("initialize", peer.offer.clone())
        .unwrap();
    let offer = peer.take();
    let extension = peer
        .connection
        .send_for(Version::V2, "_early", raw("{}"))
        .unwrap();
    let early = peer.take();
    peer.response(&offer, r#"{"protocolVersion":1,"agentCapabilities":{}}"#);
    assert!(
        matches!(peer.event().payload, Payload::Initialized(token, Version::V1, _) if token == initialize)
    );
    // A pre-negotiation request keeps its admission family when it settles later.
    peer.response(&early, "{}");
    assert!(
        matches!(peer.event().payload, Payload::Response(token, Version::V2, _) if token == extension)
    );
    let snapshot = peer.connection.snapshot().unwrap();
    assert_eq!(snapshot.requested_version, Version::V2);
    assert_eq!(snapshot.negotiated_version, Some(Version::V1));
    assert!(peer.endpoint.try_take().unwrap().is_none());
    peer.session("v1-session");
    assert_eq!(
        peer.connection
            .send("initialize", raw(r#"{"protocolVersion":1}"#)),
        Err(Fault::Busy("Initialize"))
    );
    let unsupported = configured(false);
    let token = unsupported
        .connection
        .send("initialize", unsupported.offer.clone())
        .unwrap();
    unsupported.response(&unsupported.take(), r#"{"protocolVersion":1}"#);
    assert!(
        matches!(unsupported.event().payload, Payload::RequestFailed(actual, Fault::UnsupportedVersion(1)) if actual == token)
    );
    assert!(matches!(unsupported.event().payload, Payload::Closed(_)));
}
#[test]
fn v2_optional_capabilities_use_objects_and_permissions_keep_their_own_shape() {
    let peer = configured(false);
    init(
        &peer,
        r#"{"protocolVersion":2,"info":{"name":"peer","version":"1"}}"#,
        Version::V2,
    );
    exchange(
        &peer,
        "session/new",
        r#"{"cwd":"/workspace","mcpServers":[]}"#,
        r#"{"sessionId":"s"}"#,
    );
    for (method, params, capability) in [
        ("auth/logout", "{}", "Authentication"),
        ("session/delete", r#"{"sessionId":"s"}"#, "SessionDelete"),
        (
            "session/prompt",
            r#"{"sessionId":"s","prompt":[{"type":"image","mimeType":"image/png","data":"YQ=="}]}"#,
            "PromptImage",
        ),
        (
            "session/new",
            r#"{"cwd":"/x","mcpServers":[{"type":"stdio","name":"n","command":"/bin/mcp","args":[],"env":[]}]}"#,
            "McpStdio",
        ),
    ] {
        assert_eq!(
            peer.connection.send(method, raw(params)),
            Err(Fault::UnsupportedCapability(capability))
        );
    }
    peer.feed(raw(r#"{"jsonrpc":"2.0","id":"p","method":"session/request_permission","params":{"sessionId":"s","title":"Run?","subject":{"type":"tool_call","toolCall":{"toolCallId":"t"}},"options":[{"optionId":"once","name":"Once","kind":"allow_once"}]}}"#));
    let Payload::Request(right, Version::V2, _) = peer.event().payload else {
        panic!("v2 permission");
    };
    peer.connection
        .respond(
            right,
            raw(r#"{"outcome":{"outcome":"selected","optionId":"once"}}"#),
        )
        .unwrap();
    assert!(peer.take().member("result").unwrap().is_some());
}
