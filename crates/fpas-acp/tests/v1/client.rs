use super::peer::{Peer, raw};
use fpas_acp::connection::{Fault, Lifecycle, Limits, Payload, Settings};
use fpas_acp::schema::Version;

const SETUP: &str = r#"{"sessionId":"s","modes":{"currentModeId":"ask","availableModes":[{"id":"ask","name":"Ask"},{"id":"code","name":"Code"}]},"configOptions":[{"id":"model","name":"Model","type":"select","currentValue":"small","options":[{"value":"small","name":"Small"}]},{"id":"flag","name":"Flag","type":"boolean","currentValue":false}]}"#;

#[test]
fn initialization_is_atomic_single_use_and_failed_negotiation_never_becomes_ready() {
    let peer = Peer::new();
    assert_eq!(
        peer.connection
            .send("session/new", raw(r#"{"cwd":"/x","mcpServers":[]}"#)),
        Err(Fault::Busy("Initialize"))
    );
    assert_eq!(
        peer.connection
            .send("initialize", raw(r#"{"protocolVersion":2}"#)),
        Err(Fault::UnsupportedVersion(2))
    );
    assert_eq!(
        peer.connection.send(
            "initialize",
            raw(r#"{"protocolVersion":1,"clientCapabilities":{"terminal":true}}"#)
        ),
        Err(Fault::InvalidArgument("InitializationOffer"))
    );
    assert!(peer.endpoint.try_take().unwrap().is_none());
    let token = peer
        .connection
        .send("initialize", peer.offer.clone())
        .unwrap();
    assert_eq!(peer.connection.state(), Ok(Lifecycle::Initializing));
    assert_eq!(
        peer.connection.send("initialize", peer.offer.clone()),
        Err(Fault::Busy("Initialize"))
    );
    peer.response(&peer.take(), r#"{"protocolVersion":1,"agentInfo":{"name":"peer","version":"1"},"authMethods":[{"id":"token","name":"Token"}]}"#);
    assert!(matches!(peer.event().payload, Payload::Response(actual, _, _) if actual == token));
    assert_eq!(peer.connection.state(), Ok(Lifecycle::Ready));
    assert!(
        peer.connection
            .snapshot()
            .unwrap()
            .initialization
            .unwrap()
            .text()
            .contains("authMethods")
    );
    assert_eq!(
        peer.connection.send("initialize", peer.offer.clone()),
        Err(Fault::Busy("Initialize"))
    );

    for response in [
        r#""result":{"protocolVersion":2}"#,
        r#""error":{"code":-32603,"message":"initialize failed"}"#,
    ] {
        let peer = Peer::new();
        let token = peer
            .connection
            .send("initialize", peer.offer.clone())
            .unwrap();
        let id = peer.take().member("id").unwrap().unwrap();
        peer.feed(raw(&format!(
            r#"{{"jsonrpc":"2.0","id":{},{response}}}"#,
            id.text()
        )));
        let result = peer.event();
        assert!(matches!(result.payload, Payload::RequestFailed(actual, _) if actual == token));
        assert!(matches!(peer.event().payload, Payload::Closed(status) if status.fault.is_some()));
        assert_eq!(peer.connection.state(), Ok(Lifecycle::Failed));
        assert!(peer.connection.snapshot().unwrap().initialization.is_none());
    }
}

#[test]
fn all_stable_methods_preserve_session_fields_paging_and_replay_before_load_response() {
    let mut settings = Settings::for_version(Version::V1);
    settings.client_capabilities.auth.terminal = true;
    let peer = Peer::configured(settings, Limits::default());
    peer.initialize(r#"{"protocolVersion":1,"agentCapabilities":{"loadSession":true,"auth":{"logout":{}},"mcpCapabilities":{"http":true,"sse":true},"sessionCapabilities":{"list":{},"delete":{},"resume":{},"close":{},"additionalDirectories":{}}},"authMethods":[{"id":"token","name":"Token"},{"id":"interactive","name":"Interactive","type":"terminal","args":[],"env":{}}] }"#);
    peer.exchange("authenticate", r#"{"methodId":"token"}"#, "{}");
    assert_eq!(
        peer.connection
            .send("authenticate", raw(r#"{"methodId":"missing"}"#)),
        Err(Fault::InvalidArgument("AuthMethodId"))
    );
    assert_eq!(
        peer.connection
            .send("authenticate", raw(r#"{"methodId":"interactive"}"#)),
        Err(Fault::UnsupportedCapability("TerminalAuthentication"))
    );
    peer.exchange("logout", "{}", "{}");
    peer.exchange("session/new", r#"{"cwd":"/workspace","additionalDirectories":["/other"],"mcpServers":[{"name":"stdio","command":"mcp","args":[],"env":[]},{"type":"http","name":"http","url":"https://example.test","headers":[]},{"type":"sse","name":"sse","url":"https://example.test","headers":[]}]}"#, SETUP);
    peer.exchange(
        "session/set_mode",
        r#"{"sessionId":"s","modeId":"code"}"#,
        "{}",
    );
    assert_eq!(
        peer.connection.send(
            "session/set_mode",
            raw(r#"{"sessionId":"s","modeId":"invented"}"#)
        ),
        Err(Fault::InvalidArgument("ModeId"))
    );
    assert_eq!(
        peer.connection.send(
            "session/set_config_option",
            raw(r#"{"sessionId":"s","configId":"model","value":"invented"}"#)
        ),
        Err(Fault::InvalidArgument("ConfigValue"))
    );
    peer.exchange("session/set_config_option", r#"{"sessionId":"s","configId":"flag","type":"boolean","value":true}"#, r#"{"configOptions":[{"id":"model","name":"Model","type":"select","currentValue":"small","options":[{"group":"g","name":"Models","options":[{"value":"small","name":"Small"}]}]}]}"#);
    peer.exchange(
        "session/set_config_option",
        r#"{"sessionId":"s","configId":"model","value":"small"}"#,
        r#"{"configOptions":[]}"#,
    );
    peer.exchange(
        "session/list",
        r#"{"cursor":"page 1","cwd":"/workspace"}"#,
        r#"{"sessions":[{"sessionId":"saved","cwd":"/workspace"}],"nextCursor":"page 2"}"#,
    );
    peer.exchange(
        "session/list",
        r#"{"cursor":"page 2"}"#,
        r#"{"sessions":[]}"#,
    );
    assert!(peer.endpoint.try_take().unwrap().is_none());
    let load = peer
        .connection
        .send(
            "session/load",
            raw(r#"{"sessionId":"loaded","cwd":"/workspace","mcpServers":[]}"#),
        )
        .unwrap();
    let request = peer.take();
    let replay = peer.update(
        "loaded",
        r#"{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"historical"}}"#,
    );
    peer.response(&request, "{}");
    assert_eq!(peer.event().raw, Some(replay));
    assert!(matches!(peer.event().payload, Payload::Response(token, _, _) if token == load));
    peer.exchange(
        "session/resume",
        r#"{"sessionId":"resumed","cwd":"/workspace","additionalDirectories":["/other"]}"#,
        "{}",
    );
    peer.exchange(
        "session/prompt",
        r#"{"sessionId":"loaded","prompt":[]}"#,
        r#"{"stopReason":"end_turn"}"#,
    );
    peer.exchange("session/close", r#"{"sessionId":"loaded"}"#, "{}");
    assert_eq!(
        peer.connection.send(
            "session/prompt",
            raw(r#"{"sessionId":"loaded","prompt":[]}"#)
        ),
        Err(Fault::InvalidArgument("SessionId"))
    );
    peer.exchange("session/delete", r#"{"sessionId":"loaded"}"#, "{}");
    peer.exchange(
        "session/load",
        r#"{"sessionId":"loaded","cwd":"/workspace","mcpServers":[]}"#,
        "{}",
    );
}

#[test]
fn absent_null_and_false_capabilities_reject_before_publication_but_required_methods_work() {
    for caps in [
        "{}",
        r#"{"loadSession":null,"auth":{"logout":false},"sessionCapabilities":{"list":false,"delete":false,"resume":false,"close":false}}"#,
        r#"{"loadSession":false,"auth":{"logout":null},"sessionCapabilities":{"list":null,"delete":null,"resume":null,"close":null},"promptCapabilities":{"audio":false,"image":false},"mcpCapabilities":{"http":false,"sse":false}}"#,
    ] {
        let peer = Peer::new();
        peer.initialize(&format!(
            r#"{{"protocolVersion":1,"agentCapabilities":{caps}}}"#
        ));
        peer.session("s");
        for (method, params, name) in [
            ("logout", "{}", "Logout"),
            ("session/list", "{}", "SessionList"),
            ("session/delete", r#"{"sessionId":"s"}"#, "SessionDelete"),
            ("session/close", r#"{"sessionId":"s"}"#, "SessionClose"),
            (
                "session/load",
                r#"{"sessionId":"saved","cwd":"/x","mcpServers":[]}"#,
                "SessionLoad",
            ),
            (
                "session/resume",
                r#"{"sessionId":"saved","cwd":"/x"}"#,
                "SessionResume",
            ),
            (
                "session/set_mode",
                r#"{"sessionId":"s","modeId":"x"}"#,
                "SessionModes",
            ),
            (
                "session/set_config_option",
                r#"{"sessionId":"s","configId":"model","value":"x"}"#,
                "SessionConfigOption",
            ),
            (
                "session/new",
                r#"{"cwd":"/x","mcpServers":[],"additionalDirectories":["/other"]}"#,
                "AdditionalDirectories",
            ),
            (
                "session/new",
                r#"{"cwd":"/x","mcpServers":[{"type":"http","name":"n","url":"https://example.test","headers":[]}]}"#,
                "McpHttp",
            ),
            (
                "session/new",
                r#"{"cwd":"/x","mcpServers":[{"type":"sse","name":"n","url":"https://example.test","headers":[]}]}"#,
                "McpSse",
            ),
            (
                "session/prompt",
                r#"{"sessionId":"s","prompt":[{"type":"image","data":"YQ==","mimeType":"image/png"}]}"#,
                "PromptImage",
            ),
            (
                "session/prompt",
                r#"{"sessionId":"s","prompt":[{"type":"audio","data":"YQ==","mimeType":"audio/wav"}]}"#,
                "PromptAudio",
            ),
            (
                "session/prompt",
                r#"{"sessionId":"s","prompt":[{"type":"resource","resource":{"uri":"file:///x","text":"context"}}]}"#,
                "PromptEmbeddedContext",
            ),
        ] {
            assert_eq!(
                peer.connection.send(method, raw(params)),
                Err(Fault::UnsupportedCapability(name)),
                "{method}"
            );
        }
        assert!(peer.endpoint.try_take().unwrap().is_none());
        peer.exchange("session/prompt", r#"{"sessionId":"s","prompt":[{"type":"text","text":"hello"},{"type":"resource_link","uri":"file:///x","name":"x"}]}"#, r#"{"stopReason":"end_turn"}"#);
    }
}

#[test]
fn session_slots_are_reserved_before_send_and_returned_by_setup_errors_and_close() {
    let peer = Peer::configured(
        Settings::for_version(Version::V1),
        Limits {
            sessions: 1,
            ..Limits::default()
        },
    );
    peer.initialize(
        r#"{"protocolVersion":1,"agentCapabilities":{"sessionCapabilities":{"close":{}}},"authMethods":[{"id":"token","name":"Token"}]}"#,
    );
    let token = peer
        .connection
        .send("session/new", raw(r#"{"cwd":"/x","mcpServers":[]}"#))
        .unwrap();
    assert_eq!(
        peer.connection
            .send("session/new", raw(r#"{"cwd":"/x","mcpServers":[]}"#)),
        Err(Fault::LimitExceeded("Sessions", 1))
    );
    let request = peer.take();
    let id = request.member("id").unwrap().unwrap();
    peer.feed(raw(&format!(r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32000,"message":"Authentication required"}}}}"#, id.text())));
    assert!(
        matches!(peer.event().payload, Payload::RequestFailed(actual, Fault::Peer(_)) if actual == token)
    );
    peer.exchange("authenticate", r#"{"methodId":"token"}"#, "{}");
    peer.session("s");
    peer.exchange("session/close", r#"{"sessionId":"s"}"#, "{}");
    peer.session("replacement");
}
