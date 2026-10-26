#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail at the observable boundary"
)]
//! Real SDK requests protect validation, detached dispatch and terminal ownership.

mod support;
use fpas_mcp::{
    Catalog, Fault, Limits, Server,
    model::{CallToolResult, ContentBlock},
};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use support::{Peer, object, receive, result, server, tool};

#[tokio::test]
async fn discovery_validation_callback_context_and_output_share_one_catalog() {
    let server = server(Limits::default());
    let endpoint = server.endpoint().unwrap();
    let mut peer = Peer::open(endpoint.clone()).await;
    let mut reader = server.reader().unwrap();
    peer.request(1, "tools/list", json!({})).await;
    let listed = peer.response().await;
    assert_eq!(listed["result"]["tools"][0]["name"], "update");
    assert!(listed["result"].get("nextCursor").is_none());

    for (index, arguments) in [
        json!({}),
        json!({"value":"wrong"}),
        json!({"value":-1}),
        json!({"value":1,"extra":true}),
    ]
    .into_iter()
    .enumerate()
    {
        peer.call(10 + index as u64, arguments).await;
        let reply = peer.response().await;
        assert_eq!(reply["result"]["isError"], true);
        assert!(reader.try_receive().unwrap().is_none());
    }
    peer.request(20, "tools/call", json!({"name":"missing","arguments":{}}))
        .await;
    assert_eq!(peer.response().await["error"]["code"], -32602);
    peer.request(
        21,
        "tools/call",
        json!({"name":"update","arguments":{"value":42},"_meta":{"correlation":"original"}}),
    )
    .await;
    let call = receive(&mut reader).await;
    assert_eq!(call.name, "update");
    assert_eq!(call.handler, 7);
    assert_eq!(call.arguments["value"], 42);
    assert_eq!(call.protocol_version.unwrap().as_str(), "2025-06-18");
    assert_eq!(
        serde_json::to_value(call.metadata).unwrap()["correlation"],
        "original"
    );
    assert_eq!(call.responder.endpoint(), Ok(endpoint.id()));
    let alias = call.responder.clone();
    assert_eq!(
        alias.respond(Ok(result(json!({"value":"wrong"})))),
        Err(Fault::InvalidResult)
    );
    call.responder
        .respond(Ok(result(json!({"value":42}))))
        .unwrap();
    let reply = peer.response().await;
    assert_eq!(reply["id"], 21);
    assert_eq!(reply["result"]["structuredContent"], json!({"value":42}));
    assert_eq!(
        alias.respond(Ok(result(json!({"value":43})))),
        Err(Fault::Closed)
    );
    assert!(alias.is_cancelled());
}

#[tokio::test]
async fn pending_capacity_is_held_after_receive_and_released_by_reply_or_abandonment() {
    let server = server(Limits {
        pending_calls: 1,
        ..Limits::default()
    });
    let mut peer = Peer::open(server.endpoint().unwrap()).await;
    let mut reader = server.reader().unwrap();
    peer.call(1, json!({"value":1})).await;
    let call = receive(&mut reader).await;
    peer.call(2, json!({"value":2})).await;
    let rejected = peer.response().await;
    assert_eq!(rejected["id"], 2);
    assert!(rejected.get("error").is_some());
    call.responder
        .respond(Ok(result(json!({"value":1}))))
        .unwrap();
    assert_eq!(peer.response().await["id"], 1);
    peer.call(3, json!({"value":3})).await;
    drop(receive(&mut reader).await);
    assert!(peer.response().await.get("error").is_some());
    peer.call(4, json!({"value":4})).await;
    let call = receive(&mut reader).await;
    call.responder
        .respond(Ok(result(json!({"value":4}))))
        .unwrap();
    assert_eq!(
        peer.response().await["result"]["structuredContent"]["value"],
        4
    );
}

#[tokio::test]
async fn retiring_an_endpoint_clears_queued_work_and_never_revives_old_rights() {
    let server = server(Limits {
        pending_calls: 2,
        ..Limits::default()
    });
    let endpoint = server.endpoint().unwrap();
    let other = server.endpoint().unwrap();
    let mut peer = Peer::open(endpoint.clone()).await;
    let mut independent = Peer::open(other).await;
    let mut reader = server.reader().unwrap();
    peer.call(1, json!({"value":1})).await;
    let claimed = receive(&mut reader).await;
    peer.call(2, json!({"value":2})).await;
    peer.call(3, json!({"value":3})).await;
    let full = peer.response().await;
    assert_eq!(full["id"], 3);
    assert!(full.get("error").is_some());
    assert!(endpoint.close());
    assert!(!endpoint.close());
    assert!(reader.try_receive().unwrap().is_none());
    assert!(claimed.responder.is_cancelled());
    let replacement = server.endpoint().unwrap();
    assert_ne!(replacement.id(), endpoint.id());
    assert_eq!(
        claimed.responder.respond(Ok(result(json!({"value":1})))),
        Err(Fault::Closed)
    );
    independent.call(1, json!({"value":99})).await;
    reader
        .receive()
        .await
        .unwrap()
        .responder
        .respond(Ok(result(json!({"value":99}))))
        .unwrap();
    assert_eq!(
        independent.response().await["result"]["structuredContent"]["value"],
        99
    );
}

#[tokio::test]
async fn cancellation_invalidates_claimed_right_and_returns_admission_capacity() {
    let server = server(Limits {
        pending_calls: 1,
        ..Limits::default()
    });
    let mut peer = Peer::open(server.endpoint().unwrap()).await;
    let mut reader = server.reader().unwrap();
    peer.call(1, json!({"value":1})).await;
    let call = receive(&mut reader).await;
    peer.send(json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1}}))
        .await;
    tokio::time::timeout(Duration::from_secs(5), call.responder.cancelled())
        .await
        .unwrap();
    assert_eq!(
        call.responder.respond(Ok(result(json!({"value":1})))),
        Err(Fault::Closed)
    );
    peer.call(2, json!({"value":2})).await;
    let next = receive(&mut reader).await;
    next.responder
        .respond(Ok(result(json!({"value":2}))))
        .unwrap();
    loop {
        let reply = peer.response().await;
        if reply["id"] == 2 {
            assert_eq!(reply["result"]["structuredContent"]["value"], 2);
            break;
        }
    }
}

#[tokio::test]
async fn exact_large_integers_and_tool_errors_remain_distinct_from_protocol_errors() {
    let mut catalog = Catalog::new(Limits::default()).unwrap();
    let mut definition = tool();
    definition.input_schema = Arc::new(object(serde_json::from_str(r#"{"type":"object","properties":{"value":{"type":"integer","minimum":9007199254740993,"maximum":18446744073709551615}},"required":["value"]}"#).unwrap()));
    catalog.register(definition, 7).unwrap();
    let server = Server::new(catalog);
    let mut peer = Peer::open(server.endpoint().unwrap()).await;
    let mut reader = server.reader().unwrap();
    peer.call(1, json!({"value":9007199254740992u64})).await;
    assert_eq!(peer.response().await["result"]["isError"], true);
    peer.call(2, json!({"value":u64::MAX})).await;
    let call = receive(&mut reader).await;
    assert_eq!(call.arguments["value"].as_u64(), Some(u64::MAX));
    call.responder
        .respond(Ok(result(json!({"value":u64::MAX}))))
        .unwrap();
    assert_eq!(
        peer.response().await["result"]["structuredContent"]["value"].as_u64(),
        Some(u64::MAX)
    );
    peer.call(3, json!({"value":u64::MAX})).await;
    reader
        .receive()
        .await
        .unwrap()
        .responder
        .respond(Ok(CallToolResult::error(vec![ContentBlock::text(
            "application denied this update",
        )])))
        .unwrap();
    let denied = peer.response().await;
    assert_eq!(denied["result"]["isError"], true);
    assert!(denied.get("error").is_none());
}

#[tokio::test]
async fn limits_and_close_remain_live_with_an_owned_reader() {
    let server = server(Limits {
        endpoints: 1,
        message_bytes: 512,
        pending_bytes: 4096,
        pending_calls: 1,
        ..Limits::default()
    });
    let endpoint = server.endpoint().unwrap();
    assert!(matches!(
        server.endpoint(),
        Err(Fault::LimitExceeded("Endpoints", 1))
    ));
    let mut reader = server.reader().unwrap();
    assert!(matches!(server.reader(), Err(Fault::Busy)));
    let mut peer = Peer::open(endpoint).await;
    peer.request(
        1,
        "tools/call",
        json!({"name":"update","arguments":{"value":1},"_meta":{"large":"x".repeat(1024)}}),
    )
    .await;
    assert!(peer.response().await.get("error").is_some());
    assert!(reader.try_receive().unwrap().is_none());
    peer.call(2, json!({"value":2})).await;
    let call = receive(&mut reader).await;
    assert_eq!(
        call.responder
            .respond(Ok(CallToolResult::error(vec![ContentBlock::text(
                "x".repeat(1024)
            )]))),
        Err(Fault::LimitExceeded("MessageBytes", 512))
    );
    call.responder
        .respond(Ok(result(json!({"value":2}))))
        .unwrap();
    assert_eq!(peer.response().await["id"], 2);
    peer.call(3, json!({"value":3})).await;
    let pending = receive(&mut reader).await;
    assert!(server.close());
    assert!(pending.responder.is_cancelled());
    assert_eq!(
        pending.responder.respond(Ok(result(json!({"value":3})))),
        Err(Fault::Closed)
    );
    assert!(!server.close());
    assert!(matches!(reader.receive().await, Err(Fault::Closed)));
}

#[tokio::test]
async fn retained_request_context_limit_rejects_work_without_consuming_capacity() {
    let server = server(Limits {
        message_bytes: 8192,
        pending_bytes: 2048,
        ..Limits::default()
    });
    let mut peer = Peer::open(server.endpoint().unwrap()).await;
    let mut reader = server.reader().unwrap();
    for (id, metadata) in [
        (json!(1), json!({"large":"x".repeat(1024)})),
        (json!("request".repeat(512)), json!({})),
    ] {
        peer.send(
            json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{
                "name":"update","arguments":{"value":1},"_meta":metadata
            }}),
        )
        .await;
        let response = tokio::select! {
            response = peer.response() => response,
            _ = reader.receive() => panic!("retained request context exceeded its byte budget"),
        };
        assert!(response.get("error").is_some());
        assert!(reader.try_receive().unwrap().is_none());
    }
    peer.call(2, json!({"value":2})).await;
    receive(&mut reader)
        .await
        .responder
        .respond(Ok(result(json!({"value":2}))))
        .unwrap();
    assert_eq!(
        peer.response().await["result"]["structuredContent"]["value"],
        2
    );
}

#[tokio::test]
async fn pending_attachments_reject_calls_and_bound_context_cannot_be_redirected() {
    let server = server(Limits::default());
    let endpoint = server.endpoint().unwrap();
    endpoint.begin_attachment(41).unwrap();
    let mut peer = Peer::open(endpoint.clone()).await;
    let mut reader = server.reader().unwrap();
    peer.call(1, json!({"value":1})).await;
    assert!(peer.response().await.get("error").is_some());
    assert!(reader.try_receive().unwrap().is_none());
    endpoint.bind_session("trusted-session").unwrap();
    peer.request(
        2,
        "tools/call",
        json!({
            "name":"update","arguments":{"value":2},"_meta":{"sessionId":"attacker-session"}
        }),
    )
    .await;
    let call = receive(&mut reader).await;
    assert_eq!(call.connection_id, Some(41));
    assert_eq!(call.session_id.as_deref(), Some("trusted-session"));
    assert_eq!(
        endpoint.bind_session("other-session"),
        Err(Fault::InvalidArgument("Attachment"))
    );
    call.responder
        .respond(Ok(result(json!({"value":2}))))
        .unwrap();
    assert!(peer.response().await.get("result").is_some());
    endpoint.close();
    assert_eq!(endpoint.bind_session("trusted-session"), Err(Fault::Closed));
    server.close();
}
