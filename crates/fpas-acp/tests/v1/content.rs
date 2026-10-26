use super::peer::{Peer, raw};
use fpas_acp::connection::{Payload, Settings};
use fpas_acp::schema::Version;

#[test]
fn content_and_updates_preserve_original_data_without_completing_the_active_prompt() {
    let peer = Peer::new();
    peer.initialize(r#"{"protocolVersion":1,"agentCapabilities":{"promptCapabilities":{"image":true,"audio":true,"embeddedContext":true}}}"#);
    peer.session("s");
    let prompt = r#"{"sessionId":"s","prompt":[{"type":"text","text":"hello","_meta":{"exact":18446744073709551615}},{"type":"image","data":"YQ==","mimeType":"image/png"},{"type":"audio","data":"YQ==","mimeType":"audio/wav"},{"type":"resource","resource":{"uri":"file:///x","text":"context"}},{"type":"resource_link","uri":"file:///x","name":"x"}]}"#;
    let token = peer.connection.send("session/prompt", raw(prompt)).unwrap();
    let request = peer.take();
    assert_eq!(request.member("params").unwrap().unwrap(), raw(prompt));
    let updates = [
        r#"{"sessionUpdate":"user_message_chunk","messageId":"u","content":{"type":"text","text":"user"}}"#,
        r#"{"sessionUpdate":"agent_message_chunk","messageId":"a","content":{"type":"image","data":"YQ==","mimeType":"image/png"}}"#,
        r#"{"sessionUpdate":"agent_message_chunk","content":{"type":"audio","data":"YQ==","mimeType":"audio/wav"}}"#,
        r#"{"sessionUpdate":"agent_message_chunk","content":{"type":"resource","resource":{"uri":"file:///x","blob":"YQ==","mimeType":"application/octet-stream"}}}"#,
        r#"{"sessionUpdate":"agent_thought_chunk","content":{"type":"resource_link","uri":"file:///x","name":"x"}}"#,
        r#"{"sessionUpdate":"tool_call","toolCallId":"t","title":"Tool","content":[{"type":"diff","path":"/x","oldText":null,"newText":"new"},{"type":"terminal","terminalId":"term"}]}"#,
        r#"{"sessionUpdate":"tool_call_update","toolCallId":"t","title":null,"rawOutput":{"exact":1e400}}"#,
        r#"{"sessionUpdate":"tool_call_update","toolCallId":"t","title":"replacement","content":[]}"#,
        r#"{"sessionUpdate":"plan","entries":[]}"#,
        r#"{"sessionUpdate":"available_commands_update","availableCommands":[]}"#,
        r#"{"sessionUpdate":"current_mode_update","currentModeId":"ask"}"#,
        r#"{"sessionUpdate":"config_option_update","configOptions":[{"id":"fresh","name":"Fresh","type":"boolean","currentValue":true}]}"#,
        r#"{"sessionUpdate":"session_info_update","title":"New title"}"#,
        r#"{"sessionUpdate":"usage_update","size":100,"used":5}"#,
        r#"{"sessionUpdate":"notice","severity":"info","title":"Notice"}"#,
        r#"{"sessionUpdate":"compaction_update","compactionId":"c","trigger":"manual","status":"in_progress"}"#,
        r#"{"sessionUpdate":"compaction_summary_chunk","compactionId":"c","content":{"type":"text","text":"part"}}"#,
        r#"{"sessionUpdate":"compaction_summary_chunk","compactionId":"c","content":{"type":"text","text":"part"}}"#,
        r#"{"sessionUpdate":"future_update","opaque":{"number":18446744073709551615}}"#,
    ];
    for update in updates {
        let expected = peer.update("s", update);
        let event = peer.event();
        assert_eq!(event.raw, Some(expected), "{update}: {:?}", event.payload);
        assert!(
            matches!(event.payload, Payload::Notification(_, _)),
            "{update}"
        );
    }
    peer.exchange(
        "session/set_config_option",
        r#"{"sessionId":"s","configId":"fresh","type":"boolean","value":false}"#,
        r#"{"configOptions":[]}"#,
    );
    peer.response(&request, r#"{"stopReason":"end_turn"}"#);
    assert!(matches!(peer.event().payload, Payload::Response(actual, _, _) if actual == token));
}

#[test]
fn initialization_accepts_only_advertised_elicitation_modes_and_defers_ordinary_host_requests() {
    for caps in [
        "{}",
        r#"{"elicitation":null}"#,
        r#"{"elicitation":{}}"#,
        r#"{"elicitation":{"form":{}}}"#,
        r#"{"elicitation":{"url":{}}}"#,
    ] {
        let mut settings = Settings::for_version(Version::V1);
        settings.client_capabilities = fpas_acp::codec::decode(raw(caps)).unwrap().value;
        settings.client_capabilities.fs.read_text_file = true;
        let peer = Peer::configured(settings, Default::default());
        let init = peer
            .connection
            .send("initialize", peer.offer.clone())
            .unwrap();
        let request = peer.take();
        let id = request.member("id").unwrap().unwrap();
        for (mode, fields) in [
            (
                "form",
                r#""requestedSchema":{"type":"object","properties":{}}"#,
            ),
            (
                "url",
                r#""elicitationId":"e","url":"https://example.test/auth""#,
            ),
        ] {
            peer.feed(raw(&format!(r#"{{"jsonrpc":"2.0","id":"{mode}","method":"elicitation/create","params":{{"mode":"{mode}","requestId":{},"message":"Question",{fields}}}}}"#, id.text())));
            if caps.contains(mode) {
                let Payload::Request(right, _, _) = peer.event().payload else {
                    panic!("elicitation request");
                };
                peer.connection
                    .respond(right, raw(r#"{"action":"cancel"}"#))
                    .unwrap();
                assert!(peer.take().member("result").unwrap().is_some());
            } else {
                assert!(peer.take().member("error").unwrap().is_some());
            }
        }
        peer.feed(raw(r#"{"jsonrpc":"2.0","id":"unknown-scope","method":"elicitation/create","params":{"mode":"form","requestId":"absent","message":"Question","requestedSchema":{"type":"object","properties":{}}}}"#));
        assert!(peer.take().member("error").unwrap().is_some());
        peer.feed(raw(r#"{"jsonrpc":"2.0","id":"fs","method":"fs/read_text_file","params":{"sessionId":"s","path":"/x"}}"#));
        assert!(peer.take().member("error").unwrap().is_some());
        peer.response(&request, r#"{"protocolVersion":1}"#);
        assert!(matches!(peer.event().payload, Payload::Response(actual, _, _) if actual == init));
    }
}
