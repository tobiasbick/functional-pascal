use std::cell::RefCell;
use std::io::Read;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use super::{ConnectMode, NetworkConnections, remaining};

#[test]
fn pre_cancellation_prevents_dns_and_handle_creation() {
    let connections = NetworkConnections::new();
    for mode in [ConnectMode::Tcp, ConnectMode::Tls] {
        assert_eq!(
            connections.connect_with_cancellation("invalid\0host", 1, 1000, mode, || true),
            Err("Network connect cancelled".into())
        );
    }
    assert!(connections.connections.lock().expect("registry").is_empty());
}

#[test]
fn cancellation_after_resolution_prevents_tcp_attempt() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.set_nonblocking(true).expect("nonblocking");
    let connections = NetworkConnections::new();
    let checks = AtomicUsize::new(0);
    assert_eq!(
        connections.connect_with_cancellation(
            "127.0.0.1",
            i64::from(listener.local_addr().expect("address").port()),
            1000,
            ConnectMode::Tcp,
            || checks.fetch_add(1, Ordering::Relaxed) >= 1
        ),
        Err("Network connect cancelled".into())
    );
    assert_eq!(
        listener.accept().expect_err("no TCP attempt").kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[test]
fn cancellation_after_tcp_success_drops_unpublished_socket() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.set_nonblocking(true).expect("nonblocking");
    let connections = NetworkConnections::new();
    let peer = RefCell::new(None);
    assert_eq!(
        connections.connect_with_cancellation(
            "127.0.0.1",
            i64::from(listener.local_addr().expect("address").port()),
            1000,
            ConnectMode::Tcp,
            || {
                if peer.borrow().is_some() {
                    return true;
                }
                match listener.accept() {
                    Ok((stream, _)) => {
                        *peer.borrow_mut() = Some(stream);
                        true
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => false,
                    Err(error) => panic!("accept failed: {error}"),
                }
            }
        ),
        Err("Network connect cancelled".into())
    );
    let mut peer = peer.into_inner().expect("accepted then dropped");
    peer.set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout");
    assert_eq!(peer.read(&mut [0]).expect("EOF"), 0);
    assert!(connections.connections.lock().expect("registry").is_empty());
}

#[test]
fn successful_tcp_connection_is_published_and_usable() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let connections = NetworkConnections::new();
    let handle = connections
        .connect_with_cancellation(
            "127.0.0.1",
            i64::from(listener.local_addr().expect("address").port()),
            1000,
            ConnectMode::Tcp,
            || false,
        )
        .expect("connect");
    connections.write(handle, b"ok").expect("write");
    let (mut peer, _) = listener.accept().expect("accept");
    peer.set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout");
    let mut bytes = [0; 2];
    peer.read_exact(&mut bytes).expect("read");
    assert_eq!(&bytes, b"ok");
    connections.close(handle).expect("close");
}

#[test]
fn application_cancellation_interrupts_tls_handshake_without_retained_handle() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind silent peer");
    let port = listener.local_addr().expect("address").port();
    let connections = Arc::new(NetworkConnections::new());
    let cancelled = Arc::new(AtomicBool::new(false));
    let worker_connections = Arc::clone(&connections);
    let worker_cancelled = Arc::clone(&cancelled);
    let (send, receive) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = worker_connections.connect_with_cancellation(
            "127.0.0.1",
            i64::from(port),
            5000,
            ConnectMode::Tls,
            || worker_cancelled.load(Ordering::Acquire),
        );
        send.send(result).expect("result");
    });
    let (mut peer, _) = listener.accept().expect("accept");
    peer.set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout");
    peer.read_exact(&mut [0]).expect("TLS client hello");
    cancelled.store(true, Ordering::Release);
    let result = receive.recv_timeout(Duration::from_secs(2));
    drop(peer.shutdown(std::net::Shutdown::Both));
    worker.join().expect("join");
    assert_eq!(
        result.expect("prompt handshake cancellation"),
        Err("Network connect cancelled".into())
    );
    assert!(connections.connections.lock().expect("registry").is_empty());
}

#[test]
fn expired_shared_budget_and_cancellation_are_distinct() {
    let expired = Instant::now();
    assert_eq!(
        remaining(expired, || false),
        Err("Network connect timed out".into())
    );
    assert_eq!(
        remaining(expired, || true),
        Err("Network connect cancelled".into())
    );
}

#[test]
fn silent_tls_peer_exhausts_budget_without_publishing_handle() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind silent peer");
    let port = listener.local_addr().expect("address").port();
    let connections = NetworkConnections::new();
    let result = connections.connect_with_cancellation(
        "127.0.0.1",
        i64::from(port),
        30,
        ConnectMode::Tls,
        || false,
    );
    assert_eq!(result, Err("Network connect timed out".into()));
    assert!(connections.connections.lock().expect("registry").is_empty());
}

#[test]
fn invalid_arguments_and_resolution_errors_remain_errors() {
    let connections = NetworkConnections::new();
    assert!(
        connections
            .connect_with_cancellation("127.0.0.1", 0, 1000, ConnectMode::Tcp, || false)
            .expect_err("port")
            .contains("TCP port")
    );
    assert!(
        connections
            .connect_with_cancellation("127.0.0.1", 1, 0, ConnectMode::Tcp, || false)
            .expect_err("timeout")
            .contains("Network timeout")
    );
    assert!(
        connections
            .connect_with_cancellation("invalid\0host", 1, 1000, ConnectMode::Tcp, || false)
            .expect_err("resolution")
            .contains("Could not resolve")
    );
}

#[test]
fn shutdown_prevents_resolution_and_publishing() {
    let connections = NetworkConnections::new();
    connections.shutdown();
    assert_eq!(
        connections.connect_with_cancellation("invalid\0host", 1, 1000, ConnectMode::Tcp, || false),
        Err("Network connect cancelled".into())
    );
}
