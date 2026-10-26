#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail directly at their observable boundary"
)]
//! Live interaction rights and validation through the production SDK endpoint.
#[path = "v1/peer.rs"]
#[expect(
    dead_code,
    reason = "The shared peer also serves v1 streaming regressions"
)]
mod peer;
use fpas_acp::{
    connection::{Fault, Payload, Settings},
    schema::Version,
};
use peer::{Peer, raw};
use std::sync::{Arc, Barrier};

fn connected() -> Peer {
    let mut settings = Settings::for_version(Version::V1);
    settings.client_capabilities = fpas_acp::codec::decode(raw(
        r#"{"terminal":true,"elicitation":{"form":{},"url":{}}}"#,
    ))
    .unwrap()
    .value;
    let peer = Peer::configured(settings, Default::default());
    peer.initialize(r#"{"protocolVersion":1}"#);
    peer.session("s");
    peer
}
#[test]
fn host_execution_is_atomic_single_use_and_does_not_consume_the_reply_right() {
    let peer = connected();
    let request = raw(r#"{"sessionId":"s","command":"unused","args":[]}"#);
    peer.feed(raw(&format!(
        r#"{{"jsonrpc":"2.0","id":"terminal","method":"terminal/create","params":{}}}"#,
        request.text()
    )));
    let Payload::Request(responder, _, _) = peer.event().payload else {
        panic!("request");
    };
    assert!(matches!(
        peer.connection.claim_host(
            responder,
            "terminal/create",
            &raw(r#"{"sessionId":"s","command":"different"}"#)
        ),
        Err(Fault::InvalidArgument("HostRequest"))
    ));
    let barrier = Arc::new(Barrier::new(2));
    let rights: Vec<_> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..2)
            .map(|_| {
                let barrier = barrier.clone();
                let connection = peer.connection.clone();
                let request = request.clone();
                scope.spawn(move || {
                    let result = connection.claim_host(responder, "terminal/create", &request);
                    barrier.wait();
                    result
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect()
    });
    assert_eq!(rights.iter().filter(|right| right.is_ok()).count(), 1);
    assert_eq!(
        rights
            .iter()
            .filter(|right| matches!(right, Err(Fault::Busy("HostExecution"))))
            .count(),
        1
    );
    assert_eq!(
        peer.connection
            .respond(responder, raw(r#"{"terminalId":"t"}"#)),
        Err(Fault::Busy("HostExecution"))
    );
    drop(rights);
    assert!(matches!(
        peer.connection
            .claim_host(responder, "terminal/create", &request),
        Err(Fault::InvalidArgument("HostExecution"))
    ));
    peer.connection
        .respond(responder, raw(r#"{"terminalId":"t"}"#))
        .unwrap();
    assert!(peer.take().member("result").unwrap().is_some());
}
#[test]
fn permission_ids_and_form_submissions_are_checked_before_the_response_right_is_consumed() {
    let peer = connected();
    peer.feed(raw(r#"{"jsonrpc":"2.0","id":"permission","method":"session/request_permission","params":{"sessionId":"s","toolCall":{"toolCallId":"t"},"options":[{"optionId":"once","name":"Once","kind":"allow_once"}]}}"#));
    let Payload::Request(permission, _, _) = peer.event().payload else {
        panic!("permission");
    };
    assert_eq!(
        peer.connection.respond(
            permission,
            raw(r#"{"outcome":{"outcome":"selected","optionId":"forged"}}"#)
        ),
        Err(Fault::InvalidArgument("PermissionOptionId"))
    );
    peer.connection
        .respond(
            permission,
            raw(r#"{"outcome":{"outcome":"selected","optionId":"once"}}"#),
        )
        .unwrap();
    peer.take();
    let request = raw(
        r#"{"mode":"form","sessionId":"s","message":"Form","requestedSchema":{"type":"object","properties":{"text":{"type":"string","minLength":1,"maxLength":5,"pattern":"^[a-z]+$","default":"ok"},"number":{"type":"number","minimum":0,"maximum":2},"integer":{"type":"integer","minimum":1,"maximum":3},"flag":{"type":"boolean","default":true},"choice":{"type":"string","oneOf":[{"const":"a","title":"A"}]},"many":{"type":"array","minItems":1,"maxItems":2,"items":{"type":"string","enum":["a","b"]}}},"required":["text","flag"]}}"#,
    );
    peer.feed(raw(&format!(
        r#"{{"jsonrpc":"2.0","id":"form","method":"elicitation/create","params":{}}}"#,
        request.text()
    )));
    let Payload::Request(form, _, _) = peer.event().payload else {
        panic!("form");
    };
    let original = peer
        .connection
        .incoming(form, "elicitation/create")
        .unwrap();
    assert_eq!(
        fpas_acp::interaction::defaults(&original).unwrap(),
        raw(r#"{"text":"ok","flag":true}"#)
    );
    for content in [
        r#"{}"#,
        r#"{"text":"UPPER","flag":true}"#,
        r#"{"text":"ok","flag":"true"}"#,
        r#"{"text":"ok","flag":true,"integer":1.5}"#,
        r#"{"text":"ok","flag":true,"number":3}"#,
        r#"{"text":"ok","flag":true,"choice":"b"}"#,
        r#"{"text":"ok","flag":true,"many":["unknown"]}"#,
        r#"{"text":"ok","flag":true,"extra":1}"#,
    ] {
        assert_eq!(
            peer.connection.respond(
                form,
                raw(&format!(r#"{{"action":"accept","content":{content}}}"#))
            ),
            Err(Fault::InvalidArgument("ElicitationContent")),
            "{content}"
        );
    }
    peer.connection.respond(form, raw(r#"{"action":"accept","content":{"text":"ok","flag":false,"number":1.5,"integer":2,"choice":"a","many":["a","b"]}}"#)).unwrap();
    peer.take();
    assert_eq!(
        peer.connection
            .validate_interaction(form, &raw(r#"{"action":"cancel"}"#)),
        Err(Fault::Closed)
    );
}
