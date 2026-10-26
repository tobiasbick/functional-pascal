#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail directly at their observable boundary"
)]
//! Opt-ins are independent from each other and from peer capabilities.
#[path = "v1/peer.rs"]
mod peer;
use fpas_acp::{
    connection::{Experimental, Fault, Payload, Settings},
    schema::Version,
};
use peer::{Peer, raw};

#[test]
fn token_usage_opt_in_exposes_exact_counters_and_preserves_disabled_data_as_raw() {
    fn contains_max(value: &fpas_acp::binding::Value) -> bool {
        use fpas_acp::binding::Value;
        match value {
            Value::Unsigned(value) => *value == u64::MAX,
            Value::Record { fields, .. } | Value::Enum { fields, .. } | Value::Array(fields) => {
                fields.iter().any(contains_max)
            }
            Value::Some(value) | Value::Replacement(value) => contains_max(value),
            _ => false,
        }
    }
    for enabled in [false, true] {
        let mut settings = Settings::for_version(Version::V1);
        settings.experimental.token_usage = enabled;
        let peer = Peer::configured(settings, Default::default());
        peer.initialize(r#"{"protocolVersion":1}"#);
        peer.session("s");
        peer.connection
            .send("session/prompt", raw(r#"{"sessionId":"s","prompt":[]}"#))
            .unwrap();
        peer.response(&peer.take(), r#"{"stopReason":"end_turn","usage":{"totalTokens":18446744073709551615,"inputTokens":18446744073709551615,"outputTokens":0}}"#);
        let event = peer.event();
        let Payload::Response(_, _, payload) = event.payload else {
            panic!("prompt response");
        };
        assert_eq!(contains_max(&payload), enabled);
        assert!(event.raw.unwrap().text().contains("18446744073709551615"));
    }
}

#[test]
fn experimental_methods_fields_and_advertisements_are_rejected_until_individually_enabled() {
    let peer = Peer::new();
    peer.initialize(r#"{"protocolVersion":1,"agentCapabilities":{"providers":{},"mcpCapabilities":{"acp":true},"sessionCapabilities":{"fork":{},"systemPrompt":{}}}}"#);
    peer.session("s");
    for (method, params, name) in [
        ("providers/list", "{}", "Providers"),
        (
            "session/fork",
            r#"{"sessionId":"s","cwd":"/x"}"#,
            "SessionFork",
        ),
        (
            "mcp/message",
            r#"{"serverId":"m","requestId":"r","method":"tools/list"}"#,
            "McpOverAcp",
        ),
        (
            "session/new",
            r#"{"cwd":"/x","mcpServers":[],"systemPrompt":{"mode":"append","content":[{"type":"text","text":"instructions"}]}}"#,
            "SessionSystemPrompt",
        ),
    ] {
        assert_eq!(
            peer.connection.send(method, raw(params)),
            Err(Fault::UnsupportedCapability(name))
        );
    }
    assert!(peer.endpoint.try_take().unwrap().is_none());
    let expected = peer.update("s", r#"{"sessionUpdate":"plan_removed","planId":"p"}"#);
    let event = peer.event();
    assert_eq!(event.raw, Some(expected));
    assert!(matches!(event.payload, Payload::UnknownNotification(_, _)));
}
#[test]
fn providers_fork_system_prompts_and_restricted_observed_sessions_follow_the_advertised_contract() {
    let mut settings = Settings::for_version(Version::V1);
    settings.experimental = Experimental {
        providers: true,
        fork: true,
        system_prompt: true,
        subagents: true,
        plans: true,
        ..Experimental::default()
    };
    settings.client_capabilities = fpas_acp::codec::decode(raw(r#"{"plan":{},"subagents":{}}"#))
        .unwrap()
        .value;
    let peer = Peer::configured(settings, Default::default());
    peer.initialize(r#"{"protocolVersion":1,"agentCapabilities":{"providers":{},"sessionCapabilities":{"fork":{},"systemPrompt":{"mode":{"override":{}}}}}}"#);
    peer.exchange("providers/list", "{}", r#"{"providers":[]}"#);
    peer.exchange("providers/set", r#"{"providerId":"private","apiType":"openai","baseUrl":"https://example.test","headers":{"Authorization":"secret-fixture"}}"#, "{}");
    peer.exchange("providers/disable", r#"{"providerId":"private"}"#, "{}");
    peer.exchange("session/new", r#"{"cwd":"/x","mcpServers":[],"systemPrompt":{"mode":"append","content":[{"type":"text","text":"instructions"}]}}"#, r#"{"sessionId":"parent"}"#);
    peer.exchange(
        "session/fork",
        r#"{"sessionId":"parent","cwd":"/fork","mcpServers":null,"systemPrompt":null}"#,
        r#"{"sessionId":"fork"}"#,
    );
    peer.exchange(
        "session/fork",
        r#"{"sessionId":"parent","cwd":"/fork2","systemPrompt":{"mode":"override","content":[]}}"#,
        r#"{"sessionId":"fork2"}"#,
    );
    for update in [
        r#"{"sessionUpdate":"plan_update","plan":{"type":"items","planId":"p","entries":[]}}"#,
        r#"{"sessionUpdate":"plan_removed","planId":"p"}"#,
        r#"{"sessionUpdate":"subagent_update","sessionId":"child"}"#,
    ] {
        let expected = peer.update("parent", update);
        let event = peer.event();
        assert_eq!(event.raw, Some(expected), "{update}: {:?}", event.payload);
        assert!(
            matches!(event.payload, Payload::Notification(_, _)),
            "{update}"
        );
    }
    assert_eq!(
        peer.connection.send(
            "session/prompt",
            raw(r#"{"sessionId":"child","prompt":[]}"#)
        ),
        Err(Fault::InvalidArgument("SessionId"))
    );
    assert_eq!(
        peer.connection.send(
            "session/set_config_option",
            raw(r#"{"sessionId":"child","configId":"x","value":"x"}"#)
        ),
        Err(Fault::InvalidArgument("SessionId"))
    );
    assert_eq!(
        peer.connection
            .send("session/delete", raw(r#"{"sessionId":"child"}"#)),
        Err(Fault::InvalidArgument("SessionId"))
    );
    for update in [
        r#"{"sessionUpdate":"subagent_update","sessionId":"child","capabilities":{"cancel":{}}}"#,
        r#"{"sessionUpdate":"subagent_update","sessionId":"child"}"#,
        r#"{"sessionUpdate":"subagent_update","sessionId":"child","capabilities":false}"#,
    ] {
        peer.update("parent", update);
        peer.event();
        assert_eq!(
            peer.connection
                .notify("session/cancel", raw(r#"{"sessionId":"child"}"#)),
            Ok(true)
        );
        assert_eq!(
            peer.take().member("method").unwrap().unwrap().text(),
            r#""session/cancel""#
        );
    }
    for capabilities in ["null", "{}", r#"{"cancel":null}"#] {
        peer.update("parent", &format!(r#"{{"sessionUpdate":"subagent_update","sessionId":"child","capabilities":{capabilities}}}"#));
        peer.event();
        assert_eq!(
            peer.connection
                .notify("session/cancel", raw(r#"{"sessionId":"child"}"#)),
            Err(Fault::InvalidArgument("SessionId"))
        );
    }
    let token = peer.connection.send("providers/set", raw(r#"{"providerId":"private","apiType":"openai","baseUrl":"https://example.test","headers":{"Authorization":"secret-fixture"}}"#)).unwrap();
    let request = peer.take();
    let id = request.member("id").unwrap().unwrap();
    peer.feed(raw(&format!(r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32000,"message":"Provider failed","data":{{"token":"secret-fixture"}}}}}}"#, id.text())));
    let event = peer.event();
    assert!(
        matches!(event.payload, Payload::RequestFailed(actual, Fault::Peer(_)) if actual == token)
    );
    assert!(!format!("{event:?}").contains("secret-fixture"));
    assert!(event.raw.unwrap().text().contains("secret-fixture"));
    peer.update(
        "fork",
        r#"{"sessionUpdate":"subagent_update","sessionId":"child","capabilities":{"cancel":{}}}"#,
    );
    assert!(
        matches!(peer.event().payload, Payload::Closed(status) if status.fault == Some(Fault::InvalidMessage))
    );
}
