//! Listener exchange, timeout and shutdown regressions.

use super::Failure;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use super::NetworkListeners;

fn unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve local port");
    listener.local_addr().expect("reserved address").port()
}

#[test]
fn listener_accepts_one_tcp_connection() {
    let port = unused_port();
    let listeners = NetworkListeners::new();
    let handle = listeners
        .listen("127.0.0.1", i64::from(port))
        .expect("listen");
    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        stream.write_all(b"ping").expect("write request");
        let mut response = [0_u8; 4];
        stream.read_exact(&mut response).expect("read response");
        response
    });

    let mut stream = listeners.accept(handle).expect("accept");
    let mut request = [0_u8; 4];
    stream.read_exact(&mut request).expect("read request");
    stream.write_all(b"pong").expect("write response");

    assert_eq!(request, *b"ping");
    assert_eq!(client.join().expect("join client"), *b"pong");
}

/// Windows accepted sockets inherit the listener's non-blocking mode; the accepted
/// connection must still block until its read timeout instead of failing with WouldBlock.
#[test]
fn accepted_tcp_connection_blocks_until_its_read_timeout() {
    let port = unused_port();
    let listeners = NetworkListeners::new();
    let handle = listeners
        .listen("127.0.0.1", i64::from(port))
        .expect("listen");
    let _client = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    let mut stream = listeners.accept(handle).expect("accept");
    stream
        .set_timeout(Some(Duration::from_millis(200)))
        .expect("read timeout");

    let started = Instant::now();
    let mut byte = [0_u8; 1];
    let error = stream.read(&mut byte).expect_err("no data arrives");

    assert!(
        started.elapsed() >= Duration::from_millis(150),
        "read returned before its timeout: {error}"
    );
}

#[test]
fn closed_listener_cannot_accept() {
    let port = unused_port();
    let listeners = NetworkListeners::new();
    let handle = listeners
        .listen("127.0.0.1", i64::from(port))
        .expect("listen");

    listeners.close(handle).expect("close");

    assert!(listeners.accept(handle).is_err());
}

#[test]
fn invalid_tls_configuration_does_not_bind_socket() {
    let port = unused_port();
    let listeners = NetworkListeners::new();

    let error = listeners
        .listen_tls(
            "127.0.0.1",
            i64::from(port),
            "missing-certificate.pem",
            "missing-private-key.pem",
            1_000,
        )
        .expect_err("missing TLS files must fail");

    assert!(matches!(
        error,
        Failure::Native {
            kind: crate::execution::vm::hosted::net::failure::NativeKind::IoFailure,
            ..
        }
    ));
    TcpListener::bind(("127.0.0.1", port))
        .expect("invalid TLS configuration must not bind the socket");
}

#[test]
fn close_interrupts_a_blocked_accept() {
    let port = unused_port();
    let listeners = Arc::new(NetworkListeners::new());
    let handle = listeners
        .listen("127.0.0.1", i64::from(port))
        .expect("listen");
    let accept_listeners = Arc::clone(&listeners);
    let (result_sender, result_receiver) = mpsc::channel();
    let accept = std::thread::spawn(move || {
        result_sender
            .send(accept_listeners.accept(handle))
            .expect("send accept result");
    });
    std::thread::sleep(Duration::from_millis(50));

    listeners.close(handle).expect("close");
    let result = result_receiver.recv_timeout(Duration::from_millis(200));
    if result.is_err() {
        drop(TcpStream::connect(("127.0.0.1", port)));
    }
    accept.join().expect("join accept");

    let error = match result.expect("close must interrupt the blocked accept") {
        Ok(_) => panic!("closed listener must not accept a connection"),
        Err(error) => error,
    };
    assert!(
        matches!(error, Failure::Closed),
        "unexpected error: {error}"
    );
}

#[test]
fn cancellation_interrupts_a_blocked_accept_without_closing_the_listener() {
    let port = unused_port();
    let listeners = Arc::new(NetworkListeners::new());
    let handle = listeners
        .listen("127.0.0.1", i64::from(port))
        .expect("listen");
    let cancelled = Arc::new(AtomicBool::new(false));
    let accept_listeners = Arc::clone(&listeners);
    let accept_cancelled = Arc::clone(&cancelled);
    let (result_sender, result_receiver) = mpsc::channel();
    let accept = std::thread::spawn(move || {
        result_sender
            .send(
                accept_listeners
                    .accept_with_cancellation(handle, || accept_cancelled.load(Ordering::Acquire)),
            )
            .expect("send accept result");
    });
    std::thread::sleep(Duration::from_millis(50));

    cancelled.store(true, Ordering::Release);
    let result = result_receiver
        .recv_timeout(Duration::from_millis(200))
        .expect("cancellation must interrupt the blocked accept");
    accept.join().expect("join accept");

    let error = match result {
        Ok(_) => panic!("cancelled accept must fail"),
        Err(error) => error,
    };
    assert_eq!(error, Failure::Cancelled);
    listeners.close(handle).expect("listener remains open");
}

#[test]
fn cancellation_after_tcp_setup_prevents_returning_the_connection() {
    let port = unused_port();
    let listeners = NetworkListeners::new();
    let handle = listeners
        .listen("127.0.0.1", i64::from(port))
        .expect("listen");
    let _client = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    let checks = std::sync::atomic::AtomicUsize::new(0);

    let result =
        listeners.accept_with_cancellation(handle, || checks.fetch_add(1, Ordering::Relaxed) >= 2);

    let error = match result {
        Ok(_) => panic!("final cancellation must win"),
        Err(error) => error,
    };
    assert_eq!(error, Failure::Cancelled);
    listeners.close(handle).expect("listener remains open");
}
