#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail at the observable boundary"
)]
//! Real sockets protect HTTP admission, attachment authority and joined shutdown.

#[expect(
    dead_code,
    reason = "The shared fixture also supplies the NDJSON peer for server.rs"
)]
mod support;

use fpas_mcp::{Fault, HttpEndpoint, HttpError, HttpOptions, Limits};
use serde_json::{Value, json};
use std::{future::Future, io, net::SocketAddr, task::Poll, time::Duration};
use support::{receive, result, server};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

struct Reply {
    status: u16,
    body: Vec<u8>,
}

impl Reply {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).expect("HTTP JSON response")
    }
}

async fn post(
    address: SocketAddr,
    authorization: Option<&str>,
    headers: &[&str],
    body: &str,
) -> io::Result<Reply> {
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut socket = TcpStream::connect(address).await?;
        let mut request = format!(
            "POST /mcp HTTP/1.1\r\nContent-Length: {}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2025-06-18\r\nConnection: close\r\n",
            body.len()
        );
        if !headers.iter().any(|header| header.starts_with("Host:")) {
            request.push_str(&format!("Host: {address}\r\n"));
        }
        if let Some(authorization) = authorization {
            request.push_str(&format!("Authorization: {authorization}\r\n"));
        }
        for header in headers {
            request.push_str(header);
            request.push_str("\r\n");
        }
        request.push_str("\r\n");
        request.push_str(body);
        socket.write_all(request.as_bytes()).await?;
        let mut bytes = Vec::new();
        socket.read_to_end(&mut bytes).await?;
        let end = bytes.windows(4).position(|value| value == b"\r\n\r\n")
            .ok_or_else(|| io::Error::other("HTTP response headers missing"))?;
        let headers = std::str::from_utf8(&bytes[..end])
            .map_err(io::Error::other)?;
        let status = headers.split_whitespace().nth(1)
            .ok_or_else(|| io::Error::other("HTTP response status missing"))?
            .parse().map_err(io::Error::other)?;
        let mut body = bytes[end + 4..].to_vec();
        if headers.to_ascii_lowercase().contains("transfer-encoding: chunked") {
            let mut decoded = Vec::new();
            while !body.is_empty() {
                let end = body.windows(2).position(|value| value == b"\r\n")
                    .ok_or_else(|| io::Error::other("HTTP chunk header missing"))?;
                let size = usize::from_str_radix(
                    std::str::from_utf8(&body[..end]).map_err(io::Error::other)?, 16
                ).map_err(io::Error::other)?;
                if size == 0 { break; }
                let payload = body.get(end + 2..end + 2 + size)
                    .ok_or_else(|| io::Error::other("HTTP chunk is incomplete"))?;
                decoded.extend_from_slice(payload);
                body.drain(..end + size + 4);
            }
            body = decoded;
        }
        Ok(Reply { status, body })
    }).await.map_err(io::Error::other)?
}

fn list() -> String {
    json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}).to_string()
}

fn call(value: i64) -> String {
    json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
        "name":"update","arguments":{"value":value}
    }})
    .to_string()
}

async fn admitted_incomplete_body(endpoint: &HttpEndpoint) -> BufReader<TcpStream> {
    let mut incomplete = BufReader::new(TcpStream::connect(endpoint.address()).await.unwrap());
    incomplete.get_mut().write_all(format!(
        "POST /mcp HTTP/1.1\r\nHost: {}\r\nAuthorization: {}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2025-06-18\r\nExpect: 100-continue\r\nContent-Length: 100\r\n\r\n",
        endpoint.address(), endpoint.authorization()
    ).as_bytes()).await.unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut line = String::new();
        incomplete.read_line(&mut line).await.unwrap();
        assert!(line.starts_with("HTTP/1.1 100 Continue"));
        loop {
            line.clear();
            incomplete.read_line(&mut line).await.unwrap();
            if line == "\r\n" {
                break;
            }
        }
    })
    .await
    .expect("first socket admitted before testing its capacity");
    incomplete
}

#[tokio::test]
async fn authenticated_http_discovery_and_calls_keep_attachment_authority_separate() {
    let server = server(Limits::default());
    let mut reader = server.reader().unwrap();
    let mut first = HttpEndpoint::bind(&server, HttpOptions::default())
        .await
        .unwrap();
    let mut second = HttpEndpoint::bind(&server, HttpOptions::default())
        .await
        .unwrap();
    for authorization in [None, Some("Bearer invalid"), Some(second.authorization())] {
        let reply = post(first.address(), authorization, &[], &list())
            .await
            .unwrap();
        assert_eq!(reply.status, 401);
    }
    let duplicate = format!("Authorization: {}", first.authorization());
    assert_eq!(
        post(
            first.address(),
            Some(first.authorization()),
            &[&duplicate],
            &list()
        )
        .await
        .unwrap()
        .status,
        401
    );
    for headers in [
        vec!["Host: attacker.invalid"],
        vec!["Origin: https://attacker.invalid"],
        vec!["Origin: null"],
    ] {
        assert_eq!(
            post(
                first.address(),
                Some(first.authorization()),
                &headers,
                &list()
            )
            .await
            .unwrap()
            .status,
            403
        );
    }
    assert!(reader.try_receive().unwrap().is_none());
    let initialize = json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{
        "protocolVersion":"2025-06-18","capabilities":{},
        "clientInfo":{"name":"independent-http-peer","version":"1"}
    }})
    .to_string();
    let reply = post(
        first.address(),
        Some(first.authorization()),
        &[],
        &initialize,
    )
    .await
    .unwrap();
    assert_eq!(reply.status, 200);
    assert_eq!(reply.json()["result"]["protocolVersion"], "2025-06-18");
    let initialized = json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string();
    assert_eq!(
        post(
            first.address(),
            Some(first.authorization()),
            &[],
            &initialized
        )
        .await
        .unwrap()
        .status,
        202
    );
    let reply = post(first.address(), Some(first.authorization()), &[], &list())
        .await
        .unwrap();
    assert_eq!(reply.json()["result"]["tools"][0]["name"], "update");
    let address = first.address();
    let authorization = first.authorization().to_owned();
    let request = tokio::spawn(async move {
        post(address, Some(&authorization), &[], &call(42))
            .await
            .unwrap()
    });
    let pending = receive(&mut reader).await;
    assert_eq!(pending.responder.endpoint().unwrap(), first.id());
    assert_eq!(pending.handler, 7);
    assert_eq!(pending.arguments["value"], 42);
    pending
        .responder
        .respond(Ok(result(json!({"value":42}))))
        .unwrap();
    assert_eq!(
        request.await.unwrap().json()["result"]["structuredContent"]["value"],
        42
    );
    first.close().await.unwrap();
    assert!(TcpStream::connect(address).await.is_err());
    assert_eq!(
        post(second.address(), Some(second.authorization()), &[], &list())
            .await
            .unwrap()
            .status,
        200
    );
    second.close().await.unwrap();
    assert!(reader.try_receive().unwrap().is_none());
    let standalone = server.endpoint().unwrap();
    standalone.close();
    server.close();
}

#[tokio::test]
async fn body_and_socket_bounds_apply_before_application_admission_and_idle_slots_expire() {
    let server = server(Limits {
        message_bytes: 512,
        ..Limits::default()
    });
    let mut reader = server.reader().unwrap();
    let options = HttpOptions {
        connections: 1,
        request_timeout: Duration::from_millis(500),
        ..HttpOptions::default()
    };
    let mut endpoint = HttpEndpoint::bind(&server, options).await.unwrap();
    let oversized = json!({"jsonrpc":"2.0","id":9,"method":"tools/call","params":{
        "name":"update","arguments":{"value":1},"_meta":{"padding":"x".repeat(512)}
    }})
    .to_string();
    let reply = post(
        endpoint.address(),
        Some(endpoint.authorization()),
        &[],
        &oversized,
    )
    .await
    .unwrap();
    assert_eq!(reply.status, 413);
    assert!(reader.try_receive().unwrap().is_none());

    let mut incomplete = admitted_incomplete_body(&endpoint).await;
    assert!(
        post(
            endpoint.address(),
            Some(endpoint.authorization()),
            &[],
            &list()
        )
        .await
        .is_err()
    );
    let mut tail = Vec::new();
    tokio::time::timeout(Duration::from_secs(5), incomplete.read_to_end(&mut tail))
        .await
        .expect("incomplete body deadline")
        .unwrap();
    assert_eq!(
        post(
            endpoint.address(),
            Some(endpoint.authorization()),
            &[],
            &list()
        )
        .await
        .unwrap()
        .status,
        200
    );
    endpoint.close().await.unwrap();
    server.close();
}

#[tokio::test]
async fn close_settles_http_calls_and_republication_never_accepts_retired_credentials() {
    let server = server(Limits {
        endpoints: 1,
        ..Limits::default()
    });
    let mut reader = server.reader().unwrap();
    let mut endpoint = HttpEndpoint::bind(&server, HttpOptions::default())
        .await
        .unwrap();
    let address = endpoint.address();
    let retired_authorization = endpoint.authorization().to_owned();
    let authorization = retired_authorization.clone();
    let request =
        tokio::spawn(async move { post(address, Some(&authorization), &[], &call(9)).await });
    let pending = receive(&mut reader).await;
    endpoint.stop();
    assert_eq!(
        pending.responder.respond(Ok(result(json!({"value":9})))),
        Err(Fault::Closed)
    );
    endpoint.close().await.unwrap();
    endpoint.close().await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(5), request)
        .await
        .expect("cancelled HTTP request completes")
        .unwrap();
    assert!(TcpStream::connect(address).await.is_err());
    let mut replacement = HttpEndpoint::bind(&server, HttpOptions::default())
        .await
        .unwrap();
    assert_eq!(
        post(
            replacement.address(),
            Some(&retired_authorization),
            &[],
            &list()
        )
        .await
        .unwrap()
        .status,
        401
    );
    assert_eq!(
        post(
            replacement.address(),
            Some(replacement.authorization()),
            &[],
            &list()
        )
        .await
        .unwrap()
        .status,
        200
    );
    server.close();
    replacement.close().await.unwrap();
    assert!(TcpStream::connect(replacement.address()).await.is_err());
}

#[tokio::test]
async fn rejected_bindings_do_not_consume_server_endpoint_capacity() {
    let server = server(Limits {
        endpoints: 1,
        ..Limits::default()
    });
    let options = HttpOptions {
        address: "0.0.0.0:0".parse().unwrap(),
        ..HttpOptions::default()
    };
    assert!(matches!(
        HttpEndpoint::bind(&server, options).await,
        Err(HttpError::Server(Fault::InvalidArgument("Address")))
    ));
    let occupied = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let options = HttpOptions {
        address: occupied.local_addr().unwrap(),
        ..HttpOptions::default()
    };
    assert!(matches!(
        HttpEndpoint::bind(&server, options).await,
        Err(HttpError::Io(_))
    ));
    let options = HttpOptions {
        address: "127.0.0.2:0".parse().unwrap(),
        ..HttpOptions::default()
    };
    let mut endpoint = HttpEndpoint::bind(&server, options).await.unwrap();
    assert_eq!(
        post(
            endpoint.address(),
            Some(endpoint.authorization()),
            &[],
            &list()
        )
        .await
        .unwrap()
        .status,
        200
    );
    assert!(matches!(
        HttpEndpoint::bind(&server, HttpOptions::default()).await,
        Err(HttpError::Server(Fault::LimitExceeded("Endpoints", 1)))
    ));
    endpoint.close().await.unwrap();
    server.close();
}
#[tokio::test]
async fn cancelling_close_keeps_ownership_of_the_socket_shutdown_result() {
    let server = server(Limits::default());
    let options = HttpOptions {
        shutdown_timeout: Duration::from_millis(100),
        ..HttpOptions::default()
    };
    let mut endpoint = HttpEndpoint::bind(&server, options).await.unwrap();
    let _incomplete = admitted_incomplete_body(&endpoint).await;
    {
        let mut closing = std::pin::pin!(endpoint.close());
        std::future::poll_fn(|context| match closing.as_mut().poll(context) {
            Poll::Pending => Poll::Ready(()),
            Poll::Ready(result) => {
                panic!("admitted incomplete socket must still be draining: {result:?}")
            }
        })
        .await;
    }
    assert_eq!(endpoint.close().await, Err(HttpError::ShutdownTimeout));
    assert!(TcpStream::connect(endpoint.address()).await.is_err());
    let mut replacement = HttpEndpoint::bind(&server, HttpOptions::default())
        .await
        .unwrap();
    replacement.close().await.unwrap();
    server.close();
}

#[tokio::test]
async fn http_disconnect_cancels_claimed_calls_and_releases_admission() {
    let server = server(Limits {
        pending_calls: 1,
        ..Limits::default()
    });
    let mut reader = server.reader().unwrap();
    let mut endpoint = HttpEndpoint::bind(&server, HttpOptions::default())
        .await
        .unwrap();
    let mut socket = TcpStream::connect(endpoint.address()).await.unwrap();
    let body = call(3);
    socket.write_all(format!(
        "POST /mcp HTTP/1.1\r\nHost: {}\r\nAuthorization: {}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2025-06-18\r\nContent-Length: {}\r\n\r\n{}",
        endpoint.address(), endpoint.authorization(), body.len(), body
    ).as_bytes()).await.unwrap();
    let abandoned = receive(&mut reader).await;
    drop(socket);
    tokio::time::timeout(Duration::from_secs(5), abandoned.responder.cancelled())
        .await
        .expect("disconnected HTTP caller cancels its response right");
    assert_eq!(
        abandoned.responder.respond(Ok(result(json!({"value":3})))),
        Err(Fault::Closed)
    );
    let address = endpoint.address();
    let authorization = endpoint.authorization().to_owned();
    let request = tokio::spawn(async move {
        post(address, Some(&authorization), &[], &call(4))
            .await
            .unwrap()
    });
    receive(&mut reader)
        .await
        .responder
        .respond(Ok(result(json!({"value":4}))))
        .unwrap();
    assert_eq!(
        request.await.unwrap().json()["result"]["structuredContent"]["value"],
        4
    );
    endpoint.close().await.unwrap();
    server.close();
}
