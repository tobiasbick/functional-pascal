#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Independent protocol fixtures fail at their observable boundary"
)]
//! The native raw transport preserves ID identity and enforces the selected ACP family.

use fpas_acp::{
    RawJson,
    connection::{Fault, Limits, Payload, Runtime},
    schema::Version,
};
use std::time::Duration;

#[tokio::test]
async fn numeric_and_string_request_ids_keep_independent_response_rights() {
    for version in [Version::V1, Version::V2] {
        let runtime = Runtime::new().unwrap();
        let (connection, endpoint) = runtime
            .connect_messages(version, Limits::default())
            .unwrap();
        let reader = connection.reader().unwrap();
        let mut responders = Vec::new();
        for id in ["7", r#""7""#, "null"] {
            if version == Version::V2 && id == "null" {
                continue;
            }
            let frame = RawJson::parse(&format!(
                r#"{{"jsonrpc":"2.0","id":{id},"method":"_request","params":{{}}}}"#
            ))
            .unwrap();
            assert!(endpoint.feed(frame).unwrap());
            let event = tokio::time::timeout(Duration::from_secs(5), reader.receive())
                .await
                .unwrap()
                .unwrap();
            let Payload::Request(responder, _, _) = event.payload else {
                panic!("request event");
            };
            responders.push((id, responder));
        }
        for (id, responder) in responders.into_iter().rev() {
            connection
                .respond(responder, RawJson::parse("{}").unwrap())
                .unwrap();
            let response = tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    if let Some(response) = endpoint.try_take().unwrap() {
                        break response;
                    }
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            assert_eq!(response.member("id").unwrap().unwrap().text(), id);
        }
        connection.close().unwrap();
    }
}

#[tokio::test]
async fn v2_rejects_null_request_and_cancellation_ids_at_the_wire_boundary() {
    for frame in [
        r#"{"jsonrpc":"2.0","id":null,"method":"_request","params":{}}"#,
        r#"{"jsonrpc":"2.0","method":"$/cancel_request","params":{"requestId":null}}"#,
    ] {
        let runtime = Runtime::new().unwrap();
        let (connection, endpoint) = runtime
            .connect_messages(Version::V2, Limits::default())
            .unwrap();
        let reader = connection.reader().unwrap();
        assert!(endpoint.feed(RawJson::parse(frame).unwrap()).unwrap());
        endpoint.close_input();
        let event = tokio::time::timeout(Duration::from_secs(5), reader.receive())
            .await
            .unwrap()
            .unwrap();
        assert!(
            matches!(event.payload, Payload::Closed(status) if status.fault == Some(Fault::InvalidMessage))
        );
    }
}
