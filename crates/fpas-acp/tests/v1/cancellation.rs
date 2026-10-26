use super::peer::{Peer, raw};
use fpas_acp::connection::{Fault, Payload};

#[test]
fn session_cancel_settles_only_its_permissions_and_waits_for_the_actual_prompt_response() {
    let peer = Peer::new();
    peer.initialize(r#"{"protocolVersion":1}"#);
    peer.session("a");
    peer.session("b");
    let prompt = raw(r#"{"sessionId":"a","prompt":[]}"#);
    let a = peer
        .connection
        .send("session/prompt", prompt.clone())
        .unwrap();
    let request_a = peer.take();
    let b = peer
        .connection
        .send("session/prompt", raw(r#"{"sessionId":"b","prompt":[]}"#))
        .unwrap();
    let request_b = peer.take();
    assert_eq!(
        peer.connection.send("session/prompt", prompt.clone()),
        Err(Fault::Busy("SessionPrompt"))
    );
    let mut rights = Vec::new();
    for session in ["a", "b"] {
        peer.feed(raw(&format!(r#"{{"jsonrpc":"2.0","id":"permission-{session}","method":"session/request_permission","params":{{"sessionId":"{session}","toolCall":{{"toolCallId":"t"}},"options":[{{"optionId":"once","name":"Once","kind":"allow_once"}}]}}}}"#)));
        let Payload::Request(right, _, _) = peer.event().payload else {
            panic!("permission request");
        };
        rights.push(right);
    }
    peer.connection
        .notify("session/cancel", raw(r#"{"sessionId":"a"}"#))
        .unwrap();
    assert_eq!(
        peer.take().member("method").unwrap().unwrap(),
        raw(r#""session/cancel""#)
    );
    assert_eq!(
        peer.connection.respond(
            rights[0],
            raw(r#"{"outcome":{"outcome":"selected","optionId":"once"}}"#)
        ),
        Err(Fault::Closed)
    );
    assert!(matches!(peer.event().payload, Payload::RequestCancelled(right) if right == rights[0]));
    let cancelled = peer.take();
    assert_eq!(
        cancelled.member("id").unwrap().unwrap(),
        raw(r#""permission-a""#)
    );
    assert_eq!(
        cancelled.member("result").unwrap().unwrap(),
        raw(r#"{"outcome":{"outcome":"cancelled"}}"#)
    );
    peer.connection
        .respond(
            rights[1],
            raw(r#"{"outcome":{"outcome":"selected","optionId":"once"}}"#),
        )
        .unwrap();
    assert_eq!(
        peer.take().member("id").unwrap().unwrap(),
        raw(r#""permission-b""#)
    );
    // A new permission after cancellation is answered without exposing a usable right.
    peer.feed(raw(r#"{"jsonrpc":"2.0","id":"late","method":"session/request_permission","params":{"sessionId":"a","toolCall":{"toolCallId":"late"},"options":[]}}"#));
    assert_eq!(
        peer.take().member("result").unwrap().unwrap(),
        raw(r#"{"outcome":{"outcome":"cancelled"}}"#)
    );
    for update in [
        r#"{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":""}}"#,
        r#"{"sessionUpdate":"tool_call_update","toolCallId":"t","status":"completed"}"#,
    ] {
        peer.update("a", update);
        assert!(matches!(peer.event().payload, Payload::Notification(_, _)));
        assert_eq!(
            peer.connection.send("session/prompt", prompt.clone()),
            Err(Fault::Busy("SessionPrompt"))
        );
    }
    assert!(peer.connection.cancel(b).unwrap());
    assert_eq!(
        peer.take().member("method").unwrap().unwrap(),
        raw(r#""$/cancel_request""#)
    );
    peer.response(&request_b, r#"{"stopReason":"end_turn"}"#);
    assert!(matches!(peer.event().payload, Payload::Response(token, _, _) if token == b));
    peer.response(&request_a, r#"{"stopReason":"cancelled"}"#);
    assert!(matches!(peer.event().payload, Payload::Response(token, _, _) if token == a));
    assert!(!peer.connection.cancel(a).unwrap());
    peer.exchange(
        "session/prompt",
        prompt.text(),
        r#"{"stopReason":"end_turn"}"#,
    );
    assert!(peer.reader.try_receive().unwrap().is_none());
}

#[test]
fn an_admitted_permission_response_wins_over_later_session_cancellation() {
    let peer = Peer::new();
    peer.initialize(r#"{"protocolVersion":1}"#);
    peer.session("a");
    peer.feed(raw(r#"{"jsonrpc":"2.0","id":"p","method":"session/request_permission","params":{"sessionId":"a","toolCall":{"toolCallId":"t"},"options":[]}}"#));
    let Payload::Request(right, _, _) = peer.event().payload else {
        panic!("permission");
    };
    peer.connection
        .respond(right, raw(r#"{"outcome":{"outcome":"cancelled"}}"#))
        .unwrap();
    peer.connection
        .notify("session/cancel", raw(r#"{"sessionId":"a"}"#))
        .unwrap();
    assert!(peer.take().member("result").unwrap().is_some());
    assert!(peer.take().member("method").unwrap().is_some());
    assert!(peer.reader.try_receive().unwrap().is_none());
}

#[test]
fn close_releases_permissions_and_late_responses_do_not_overwrite_a_reopened_session() {
    let peer = Peer::new();
    peer.initialize(r#"{"protocolVersion":1,"agentCapabilities":{"loadSession":true,"sessionCapabilities":{"close":{}}}}"#);
    peer.exchange("session/new", r#"{"cwd":"/workspace","mcpServers":[]}"#, r#"{"sessionId":"s","configOptions":[{"id":"old","name":"Old","type":"boolean","currentValue":false}]}"#);
    let config = peer
        .connection
        .send(
            "session/set_config_option",
            raw(r#"{"sessionId":"s","configId":"old","type":"boolean","value":true}"#),
        )
        .unwrap();
    let config_request = peer.take();
    let prompt = raw(r#"{"sessionId":"s","prompt":[]}"#);
    let old = peer
        .connection
        .send("session/prompt", prompt.clone())
        .unwrap();
    let request = peer.take();
    peer.feed(raw(r#"{"jsonrpc":"2.0","id":"permission","method":"session/request_permission","params":{"sessionId":"s","toolCall":{"toolCallId":"t"},"options":[]}}"#));
    let Payload::Request(right, _, _) = peer.event().payload else {
        panic!("permission");
    };
    let closing = peer
        .connection
        .send("session/close", raw(r#"{"sessionId":"s"}"#))
        .unwrap();
    let close = peer.take();
    assert_eq!(
        peer.connection.send("session/prompt", prompt.clone()),
        Err(Fault::Busy("SessionClose"))
    );
    peer.response(&close, "{}");
    assert!(matches!(peer.event().payload, Payload::Response(token, _, _) if token == closing));
    assert!(matches!(peer.event().payload, Payload::RequestCancelled(token) if token == right));
    assert_eq!(
        peer.take().member("result").unwrap().unwrap(),
        raw(r#"{"outcome":{"outcome":"cancelled"}}"#)
    );
    peer.exchange(
        "session/load",
        r#"{"sessionId":"s","cwd":"/x","mcpServers":[]}"#,
        r#"{"configOptions":[{"id":"fresh","name":"Fresh","type":"boolean","currentValue":false}]}"#,
    );
    peer.response(&config_request, r#"{"configOptions":[]}"#);
    assert!(matches!(peer.event().payload, Payload::Response(token, _, _) if token == config));
    peer.exchange(
        "session/set_config_option",
        r#"{"sessionId":"s","configId":"fresh","type":"boolean","value":true}"#,
        r#"{"configOptions":[]}"#,
    );
    let current = peer
        .connection
        .send("session/prompt", prompt.clone())
        .unwrap();
    let current_request = peer.take();
    peer.response(&request, r#"{"stopReason":"cancelled"}"#);
    assert!(matches!(peer.event().payload, Payload::Response(token, _, _) if token == old));
    assert_eq!(
        peer.connection.send("session/prompt", prompt),
        Err(Fault::Busy("SessionPrompt"))
    );
    peer.response(&current_request, r#"{"stopReason":"end_turn"}"#);
    assert!(matches!(peer.event().payload, Payload::Response(token, _, _) if token == current));
}
