//! Observable production-endpoint behavior with a deterministic peer, without network access.

use fpas_acp::{
    RawJson,
    connection::{Connection, Event, Fault, Limits, Payload, Reader, Runtime},
    schema::Version,
    transport::MessageEndpoint,
};
use std::time::Duration;

fn raw(text: &str) -> RawJson {
    RawJson::parse(text).unwrap()
}
fn wait<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap()
        .block_on(async {
            tokio::time::timeout(Duration::from_secs(10), future)
                .await
                .expect("peer protocol must progress")
        })
}
fn take(endpoint: &MessageEndpoint) -> RawJson {
    wait(async {
        loop {
            if let Some(frame) = endpoint.try_take().unwrap() {
                return frame;
            }
            assert!(
                !endpoint.output_closed(),
                "output closed before expected peer receipt"
            );
            tokio::task::yield_now().await;
        }
    })
}
fn event(reader: &Reader) -> Event {
    wait(reader.receive()).unwrap()
}
fn respond(endpoint: &MessageEndpoint, request: &RawJson, result: &str) {
    let id = request.member("id").unwrap().unwrap();
    let frame = raw(&format!(
        r#"{{"jsonrpc":"2.0","id":{},"result":{result},"extra":1e400}}"#,
        id.text()
    ));
    wait(async {
        loop {
            if endpoint.feed(frame.clone()).unwrap() {
                return;
            }
            tokio::task::yield_now().await;
        }
    });
}
fn closed(connection: &Connection, reader: &Reader) {
    assert!(connection.close().unwrap());
    assert!(!connection.close().unwrap());
    assert!(matches!(event(reader).payload, Payload::Closed(_)));
    assert_eq!(reader.try_receive(), Err(Fault::Closed));
    assert!(connection.release().unwrap());
    assert_eq!(connection.state(), Err(Fault::Closed));
}

#[test]
fn delayed_request_does_not_block_responses_notifications_or_cancellation() {
    let runtime = Runtime::new().unwrap();
    let (connection, endpoint) = runtime
        .connect_messages(Version::V1, Limits::default())
        .unwrap();
    let reader = connection.reader().unwrap();
    assert!(matches!(connection.reader(), Err(Fault::Busy("Receive"))));
    let token = connection
        .send("_remote", raw(r#"{"value":9007199254740993}"#))
        .unwrap();
    let request = take(&endpoint);
    assert_eq!(
        request.member("method").unwrap().unwrap().text(),
        r#""_remote""#
    );
    let inbound = raw(
        r#"{"jsonrpc":"2.0","id":"host","method":"_host","params":{"n":1e400},"envelope":9007199254740993}"#,
    );
    assert!(endpoint.feed(inbound.clone()).unwrap());
    respond(&endpoint, &request, r#"{"answer":18446744073709551615}"#);
    assert!(
        endpoint
            .feed(raw(
                r#"{"jsonrpc":"2.0","method":"future/event","params":{"next":true}}"#
            ))
            .unwrap()
    );
    let first = event(&reader);
    assert_eq!(first.raw, Some(inbound));
    let Payload::Request(responder, Version::V1, _) = first.payload else {
        panic!("request");
    };
    let second = event(&reader);
    assert!(matches!(second.payload, Payload::Response(actual, Version::V1, _) if actual == token));
    assert!(second.raw.unwrap().text().contains("1e400"));
    assert!(
        matches!(event(&reader).payload, Payload::UnknownNotification(method, _) if method == "future/event")
    );
    assert!(!connection.cancel(token).unwrap());
    assert!(
        endpoint
            .feed(raw(
                r#"{"jsonrpc":"2.0","method":"$/cancel_request","params":{"requestId":"host"}}"#
            ))
            .unwrap()
    );
    assert!(
        endpoint
            .feed(raw(
                r#"{"jsonrpc":"2.0","method":"after/cancel","params":{}}"#
            ))
            .unwrap()
    );
    assert!(
        matches!(event(&reader).payload, Payload::RequestCancelled(actual) if actual == responder)
    );
    assert!(
        matches!(event(&reader).payload, Payload::UnknownNotification(method, _) if method == "after/cancel")
    );
    assert_eq!(connection.respond(responder, raw("{}")), Err(Fault::Closed));
    let error = take(&endpoint);
    assert!(error.member("error").unwrap().is_some());
    closed(&connection, &reader);
}

#[test]
fn responder_validation_preserves_its_single_use_right_and_foreign_ids_fail() {
    let runtime = Runtime::new().unwrap();
    let (connection, endpoint) = runtime
        .connect_messages(Version::V1, Limits::default())
        .unwrap();
    let (foreign, _) = runtime
        .connect_messages(Version::V1, Limits::default())
        .unwrap();
    let reader = connection.reader().unwrap();
    assert!(endpoint.feed(raw(r#"{"jsonrpc":"2.0","id":1,"method":"fs/read_text_file","params":{"sessionId":"s","path":"/example"}}"#)).unwrap());
    let Payload::Request(responder, _, _) = event(&reader).payload else {
        panic!("request");
    };
    assert_eq!(foreign.responder(responder.id()), Err(Fault::Closed));
    assert!(
        connection
            .respond(responder, raw(r#"{"wrong":true}"#))
            .is_err()
    );
    assert!(
        connection
            .respond(responder, raw(r#"{"content":"correct"}"#))
            .unwrap()
    );
    assert_eq!(
        connection.respond(responder, raw(r#"{"content":"again"}"#)),
        Err(Fault::Closed)
    );
    let response = take(&endpoint);
    assert_eq!(
        response.member("result").unwrap().unwrap().text(),
        r#"{"content":"correct"}"#
    );
    closed(&connection, &reader);
}

#[test]
fn full_event_queue_keeps_terminal_settlement_outside_the_queue_budget() {
    let runtime = Runtime::new().unwrap();
    let limits = Limits {
        queue_entries: 1,
        ..Limits::default()
    };
    let (connection, endpoint) = runtime.connect_messages(Version::V1, limits).unwrap();
    let reader = connection.reader().unwrap();
    let token = connection.send("_pending", raw("{}")).unwrap();
    let _request = take(&endpoint);
    assert!(
        endpoint
            .feed(raw(r#"{"jsonrpc":"2.0","method":"first","params":{}}"#))
            .unwrap()
    );
    // Feed waits for a real input receipt; no timing-based sleeps establish capacity.
    wait(async {
        loop {
            match endpoint.feed(raw(r#"{"jsonrpc":"2.0","method":"overflow","params":{}}"#)) {
                Ok(true) => break,
                Ok(false) => tokio::task::yield_now().await,
                Err(error) => panic!("input unexpectedly closed: {error}"),
            }
        }
    });
    wait(async {
        while !connection.is_closed() {
            tokio::task::yield_now().await;
        }
    });
    assert!(
        matches!(event(&reader).payload, Payload::UnknownNotification(method, _) if method == "first")
    );
    assert!(
        matches!(event(&reader).payload, Payload::RequestFailed(actual, Fault::LimitExceeded("EventQueue", _)) if actual == token)
    );
    assert!(
        matches!(event(&reader).payload, Payload::Closed(status) if matches!(status.fault, Some(Fault::LimitExceeded("EventQueue", _))))
    );
    assert_eq!(reader.try_receive(), Err(Fault::Closed));
}

#[test]
fn eof_drains_admitted_input_then_settles_each_unanswered_request_once() {
    let runtime = Runtime::new().unwrap();
    let (connection, endpoint) = runtime
        .connect_messages(Version::V1, Limits::default())
        .unwrap();
    let reader = connection.reader().unwrap();
    let first = connection.send("_one", raw("{}")).unwrap();
    let second = connection.send("_two", raw("{}")).unwrap();
    let _ = take(&endpoint);
    let _ = take(&endpoint);
    assert!(
        endpoint
            .feed(raw(r#"{"jsonrpc":"2.0","method":"last","params":{}}"#))
            .unwrap()
    );
    assert!(endpoint.close_input());
    assert!(!endpoint.close_input());
    assert!(
        matches!(event(&reader).payload, Payload::UnknownNotification(method, _) if method == "last")
    );
    for token in [first, second] {
        assert!(
            matches!(event(&reader).payload, Payload::RequestFailed(actual, Fault::Closed) if actual == token)
        );
    }
    assert!(matches!(event(&reader).payload, Payload::Closed(status) if status.fault.is_none()));
    assert_eq!(reader.try_receive(), Err(Fault::Closed));
}

#[test]
fn blocked_output_keeps_its_admission_credit_and_rejects_before_publication() {
    let runtime = Runtime::new().unwrap();
    let limits = Limits {
        queue_entries: 1,
        outbound_entries: 1,
        ..Limits::default()
    };
    let (connection, endpoint) = runtime.connect_messages(Version::V1, limits).unwrap();
    let reader = connection.reader().unwrap();
    let _first = connection.send("_first", raw("{}")).unwrap();
    let _second = wait(async {
        loop {
            match connection.send("_second", raw("{}")) {
                Ok(token) => break token,
                Err(Fault::LimitExceeded("CommandQueue", _)) => tokio::task::yield_now().await,
                Err(error) => panic!("unexpected admission: {error}"),
            }
        }
    });
    assert!(
        endpoint
            .feed(raw(r#"{"jsonrpc":"2.0","method":"receipt","params":{}}"#))
            .unwrap()
    );
    assert!(
        matches!(event(&reader).payload, Payload::UnknownNotification(method, _) if method == "receipt")
    );
    assert!(matches!(
        connection.send("_rejected", raw("{}")),
        Err(Fault::LimitExceeded("CommandQueue", _))
    ));
    assert_eq!(
        take(&endpoint).member("method").unwrap().unwrap().text(),
        r#""_first""#
    );
    assert_eq!(
        take(&endpoint).member("method").unwrap().unwrap().text(),
        r#""_second""#
    );
    assert!(endpoint.try_take().unwrap().is_none());
    assert!(connection.close().unwrap());
    for _ in 0..2 {
        assert!(matches!(
            event(&reader).payload,
            Payload::RequestFailed(_, Fault::Closed)
        ));
    }
    assert!(matches!(event(&reader).payload, Payload::Closed(_)));
}

#[test]
fn invalid_protocol_notifications_fail_instead_of_hiding_eof_behind_a_receipt() {
    let runtime = Runtime::new().unwrap();
    let (connection, endpoint) = runtime
        .connect_messages(Version::V1, Limits::default())
        .unwrap();
    assert!(
        endpoint
            .feed(raw(
                r#"{"jsonrpc":"2.0","method":"$/cancel_request","params":{}}"#
            ))
            .unwrap()
    );
    endpoint.close_input();
    let reader = connection.reader().unwrap();
    assert!(
        matches!(event(&reader).payload, Payload::Closed(status) if status.fault == Some(Fault::InvalidMessage))
    );
}

#[test]
fn concurrent_close_and_cancel_settle_a_token_once_across_aliases() {
    let runtime = Runtime::new().unwrap();
    for _ in 0..32 {
        let (connection, endpoint) = runtime
            .connect_messages(Version::V1, Limits::default())
            .unwrap();
        let token = connection.send("_pending", raw("{}")).unwrap();
        let _receipt = take(&endpoint);
        let barrier = std::sync::Barrier::new(2);
        std::thread::scope(|scope| {
            let alias = connection.clone();
            let barrier = &barrier;
            let closing = scope.spawn(move || {
                barrier.wait();
                alias.close().unwrap()
            });
            barrier.wait();
            let _ = connection.cancel(token).unwrap();
            assert!(closing.join().unwrap());
        });
        let reader = connection.reader().unwrap();
        assert!(
            matches!(event(&reader).payload, Payload::RequestFailed(actual, Fault::Closed) if actual == token)
        );
        assert!(matches!(event(&reader).payload, Payload::Closed(_)));
        assert_eq!(reader.try_receive(), Err(Fault::Closed));
        connection.release().unwrap();
    }
}

#[test]
fn protocol_cancellation_preserves_admission_order_between_notifications() {
    let runtime = Runtime::new().unwrap();
    let (connection, endpoint) = runtime
        .connect_messages(Version::V1, Limits::default())
        .unwrap();
    let token = connection.send("_work", raw("{}")).unwrap();
    let request = take(&endpoint);
    connection.notify("_before", raw("{}")).unwrap();
    assert!(connection.cancel(token).unwrap());
    assert!(!connection.cancel(token).unwrap());
    connection.notify("_after", raw("{}")).unwrap();
    for method in ["_before", "$/cancel_request", "_after"] {
        let frame = take(&endpoint);
        assert_eq!(
            frame.member("method").unwrap().unwrap().text(),
            serde_json::to_string(method).unwrap()
        );
    }
    respond(&endpoint, &request, "{}");
    let reader = connection.reader().unwrap();
    assert!(matches!(event(&reader).payload, Payload::Response(actual, _, _) if actual == token));
    assert!(!connection.cancel(token).unwrap());
    closed(&connection, &reader);
}

#[test]
fn request_slots_are_reserved_until_response_and_returned_before_the_next_request() {
    let runtime = Runtime::new().unwrap();
    let limits = Limits {
        outgoing_requests: 1,
        incoming_requests: 1,
        ..Limits::default()
    };
    let (connection, endpoint) = runtime.connect_messages(Version::V1, limits).unwrap();
    let reader = connection.reader().unwrap();
    let first = connection.send("_first", raw("{}")).unwrap();
    let frame = take(&endpoint);
    assert!(matches!(
        connection.send("_overflow", raw("{}")),
        Err(Fault::LimitExceeded("OutgoingRequests", 1))
    ));
    respond(&endpoint, &frame, "{}");
    assert!(matches!(event(&reader).payload, Payload::Response(token, _, _) if token == first));
    let _second = connection.send("_second", raw("{}")).unwrap();
    let _ = take(&endpoint);
    assert!(
        endpoint
            .feed(raw(
                r#"{"jsonrpc":"2.0","id":1,"method":"_host","params":{}}"#
            ))
            .unwrap()
    );
    let Payload::Request(responder, _, _) = event(&reader).payload else {
        panic!("request");
    };
    connection.respond(responder, raw("{}")).unwrap();
    let _reply = take(&endpoint);
    assert!(
        endpoint
            .feed(raw(
                r#"{"jsonrpc":"2.0","id":2,"method":"_host","params":{}}"#
            ))
            .unwrap()
    );
    assert!(matches!(event(&reader).payload, Payload::Request(_, _, _)));
    assert!(
        endpoint
            .feed(raw(
                r#"{"jsonrpc":"2.0","id":3,"method":"_host","params":{}}"#
            ))
            .unwrap()
    );
    assert!(matches!(
        event(&reader).payload,
        Payload::RequestFailed(_, Fault::LimitExceeded("IncomingRequests", 1))
    ));
    assert!(
        matches!(event(&reader).payload, Payload::Closed(status) if status.fault == Some(Fault::LimitExceeded("IncomingRequests", 1)))
    );
}

#[test]
fn detached_event_bytes_fail_visibly_when_raw_input_alone_fits_its_budget() {
    let runtime = Runtime::new().unwrap();
    let limits = Limits {
        queue_bytes: 512,
        ..Limits::default()
    };
    let (connection, endpoint) = runtime.connect_messages(Version::V1, limits).unwrap();
    let prefix = r#"{"jsonrpc":"2.0","method":"future/event","params":{"text":""#;
    let suffix = r#""}}"#;
    let frame = raw(&format!(
        "{prefix}{}{suffix}",
        "x".repeat(512 - prefix.len() - suffix.len())
    ));
    assert_eq!(frame.text().len(), 512);
    assert!(endpoint.feed(frame).unwrap());
    let reader = connection.reader().unwrap();
    assert!(
        matches!(event(&reader).payload, Payload::Closed(status) if status.fault == Some(Fault::LimitExceeded("EventQueue", 512)))
    );
    assert_eq!(reader.try_receive(), Err(Fault::Closed));
}

#[test]
fn unmatched_responses_do_not_accumulate_in_sdk_staging_or_hide_known_results() {
    let runtime = Runtime::new().unwrap();
    let (connection, endpoint) = runtime
        .connect_messages(Version::V1, Limits::default())
        .unwrap();
    let token = connection.send("_known", raw("{}")).unwrap();
    let request = take(&endpoint);
    wait(async {
        for id in 0..1024 {
            let frame = raw(&format!(
                r#"{{"jsonrpc":"2.0","id":"unknown-{id}","result":{{"n":1e400}}}}"#
            ));
            loop {
                if endpoint.feed(frame.clone()).unwrap() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        }
    });
    respond(&endpoint, &request, "{}");
    let reader = connection.reader().unwrap();
    assert!(matches!(event(&reader).payload, Payload::Response(actual, _, _) if actual == token));
    assert!(reader.try_receive().unwrap().is_none());
    closed(&connection, &reader);
}
