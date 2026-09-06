use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use super::super::NetworkConnections;

mod tls;

fn connected_pair() -> (NetworkConnections, u64, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let connections = NetworkConnections::new();
    let handle = connections
        .connect_tcp(
            "127.0.0.1",
            i64::from(listener.local_addr().expect("address").port()),
            1000,
        )
        .expect("connect");
    let (peer, _) = listener.accept().expect("accept");
    (connections, handle, peer)
}

#[test]
fn cancellation_after_pending_read_preserves_connection_and_timeout() {
    let (connections, handle, mut peer) = connected_pair();
    connections.set_timeout(handle, 1000).expect("timeout");
    let checks = AtomicUsize::new(0);
    let result = connections
        .read_with_cancellation(handle, 8, || checks.fetch_add(1, Ordering::Relaxed) >= 3);
    assert_eq!(result, Err("Network read cancelled".to_string()));
    let connection = connections.connection(handle).expect("still open");
    assert_eq!(
        connection
            .transport
            .lock()
            .expect("lock")
            .read_timeout()
            .expect("timeout"),
        Some(Duration::from_secs(1))
    );
    let writer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        peer.write_all(b"next").expect("write");
    });
    assert_eq!(connections.read(handle, 8).expect("ordinary read"), b"next");
    writer.join().expect("writer");
}

#[test]
fn pre_cancelled_read_does_not_consume_available_bytes() {
    let (connections, handle, mut peer) = connected_pair();
    peer.write_all(b"x").expect("write");
    assert_eq!(
        connections.read_with_cancellation(handle, 1, || true),
        Err("Network read cancelled".to_string())
    );
    assert_eq!(
        connections
            .read_with_cancellation(handle, 1, || false)
            .expect("read"),
        b"x"
    );
}

#[test]
fn configured_timeout_bounds_cancellable_read() {
    let (connections, handle, _peer) = connected_pair();
    connections.set_timeout(handle, 30).expect("timeout");
    let started = Instant::now();
    assert_eq!(
        connections
            .read_with_cancellation(handle, 1, || started.elapsed() >= Duration::from_secs(2)),
        Err("Network read timed out".to_string())
    );
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn cancellation_is_observed_while_waiting_for_transport_lock() {
    let (connections, handle, _peer) = connected_pair();
    let connection = connections.connection(handle).expect("connection");
    let _locked = connection.transport.lock().expect("lock");
    let checks = AtomicUsize::new(0);
    assert_eq!(
        connections.read_with_cancellation(handle, 1, || {
            checks.fetch_add(1, Ordering::Relaxed) >= 2
        }),
        Err("Network read cancelled".to_string())
    );
}

#[test]
fn eof_and_invalid_size_remain_distinct_from_cancellation() {
    let (connections, handle, peer) = connected_pair();
    drop(peer);
    assert_eq!(
        connections
            .read_with_cancellation(handle, 1, || false)
            .expect("EOF"),
        Vec::<u8>::new()
    );
    assert!(
        connections
            .read_with_cancellation(handle, 0, || false)
            .expect_err("size")
            .contains("read size")
    );
}
