use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use super::super::polling::Direction;
use super::super::test_connections::{tcp_pair, tls_pair};
use super::super::{MAX_IO_BYTES, NetworkConnections, Transport};

#[test]
fn pre_cancelled_write_preserves_connection_without_sending_data() {
    let (connections, handle, mut peer) = tcp_pair();
    assert_eq!(
        connections.write_with_cancellation(handle, b"wrong", || true),
        Err("Network write cancelled".into())
    );
    assert_eq!(
        connections.write_with_cancellation(handle, b"next", || false),
        Ok(4)
    );
    let mut bytes = [0; 4];
    peer.read_exact(&mut bytes).expect("read");
    assert_eq!(&bytes, b"next");
}

#[test]
fn partial_progress_wins_over_concurrent_cancellation_without_retry() {
    let (connections, handle, _peer) = tcp_pair();
    let cancelled = AtomicBool::new(false);
    let mut attempts = 0;
    let result = connections.poll_io(
        handle,
        Direction::Write,
        || cancelled.load(Ordering::Relaxed),
        |_| {
            attempts += 1;
            cancelled.store(true, Ordering::Relaxed);
            Ok(2)
        },
    );
    assert_eq!(result, Ok(2));
    assert_eq!(attempts, 1);
}

#[test]
fn cancellation_interrupts_pending_write_attempts() {
    let (connections, handle, _peer) = tcp_pair();
    let checks = AtomicUsize::new(0);
    let mut attempts = 0;
    let result = connections.poll_io(
        handle,
        Direction::Write,
        || checks.fetch_add(1, Ordering::Relaxed) >= 3,
        |_| {
            attempts += 1;
            Err(io::ErrorKind::WouldBlock.into())
        },
    );
    assert_eq!(result, Err("Network write cancelled".into()));
    assert_eq!(attempts, 2);
}

#[test]
fn write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts() {
    let (connections, handle, _peer) = tcp_pair();
    connections.set_timeout(handle, 30).expect("timeout");
    let started = Instant::now();
    let result = connections.poll_io(
        handle,
        Direction::Write,
        || started.elapsed() >= Duration::from_secs(2),
        |_| Err(io::ErrorKind::WouldBlock.into()),
    );
    assert_eq!(result, Err("Network write timed out".into()));
    let connection = connections.connection(handle).expect("connection");
    let transport = connection.transport.lock().expect("lock");
    assert_eq!(
        transport.write_timeout().expect("timeout"),
        Some(Duration::from_millis(30))
    );
    assert_eq!(
        transport.read_timeout().expect("timeout"),
        Some(Duration::from_millis(30))
    );
}

#[test]
fn write_cancellation_interrupts_lock_contention() {
    let (connections, handle, _peer) = tcp_pair();
    let connection = connections.connection(handle).expect("connection");
    let _locked = connection.transport.lock().expect("lock");
    let checks = AtomicUsize::new(0);
    assert_eq!(
        connections
            .write_with_cancellation(handle, b"x", || checks.fetch_add(1, Ordering::Relaxed) >= 2),
        Err("Network write cancelled".into())
    );
}

#[test]
fn tcp_backpressure_can_time_out_and_cancel_without_closing() {
    let (connections, handle, _peer) = tcp_pair();
    connections.set_timeout(handle, 30).expect("timeout");
    let chunk = vec![42; 65536];
    let started = Instant::now();
    let mut accepted = 0;
    loop {
        match connections.write_with_cancellation(handle, &chunk, || {
            started.elapsed() >= Duration::from_secs(5)
        }) {
            Ok(count) => {
                assert!(count > 0);
                accepted += count;
                assert!(
                    accepted < 64 * MAX_IO_BYTES,
                    "fixture did not reach backpressure"
                );
            }
            Err(error) => {
                assert_eq!(error, "Network write timed out");
                break;
            }
        }
    }
    connections.set_timeout(handle, 1000).expect("timeout");
    let checks = AtomicUsize::new(0);
    assert_eq!(
        connections
            .write_with_cancellation(handle, &chunk, || checks.fetch_add(1, Ordering::Relaxed)
                >= 3),
        Err("Network write cancelled".into())
    );
    connections.close(handle).expect("connection remained open");
}

#[test]
fn empty_and_oversized_writes_keep_the_existing_limits() {
    let (connections, handle, _peer) = tcp_pair();
    assert_eq!(
        connections.write_with_cancellation(handle, &[], || false),
        Ok(0)
    );
    assert!(
        connections
            .write_with_cancellation(handle, &vec![0; MAX_IO_BYTES + 1], || false)
            .expect_err("size")
            .contains("write size")
    );
}

#[test]
fn tls_partial_write_reports_accepted_prefix_without_duplication() {
    let (mut client, mut peer) = tls_pair();
    client.conn.set_buffer_limit(Some(4));
    let connections = NetworkConnections::new();
    let handle = connections
        .insert_accepted(Transport::tls_client(client))
        .expect("register");
    let checks = AtomicUsize::new(0);
    assert_eq!(
        connections
            .write_with_cancellation(handle, b"abcdef", || checks.fetch_add(1, Ordering::Relaxed)
                >= 2),
        Ok(4)
    );
    assert_eq!(
        connections.write_with_cancellation(handle, b"ef", || true),
        Err("Network write cancelled".into())
    );
    assert_eq!(
        connections.write_with_cancellation(handle, b"ef", || false),
        Ok(2)
    );
    let mut bytes = [0; 6];
    peer.read_exact(&mut bytes).expect("read TLS payload");
    assert_eq!(&bytes, b"abcdef");
}

#[test]
fn accepted_tls_server_connection_supports_cancellable_write() {
    let (mut peer, server) = tls_pair();
    let connections = NetworkConnections::new();
    let handle = connections
        .insert_accepted(Transport::tls_server(server))
        .expect("register");
    assert_eq!(
        connections.write_with_cancellation(handle, b"server", || false),
        Ok(6)
    );
    let mut bytes = [0; 6];
    peer.read_exact(&mut bytes).expect("read server response");
    assert_eq!(&bytes, b"server");
}
