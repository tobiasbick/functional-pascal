use super::NetworkConnections;
use crate::vm::hosted::net::connections::test_connections::tls_pair;
use crate::vm::hosted::net::transport::Transport;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn tls_read_cancellation_preserves_session_for_subsequent_reads() {
    let (client, mut peer) = tls_pair();
    let connections = NetworkConnections::new();
    let handle = connections
        .insert_accepted(Transport::tls_client(client))
        .expect("register");
    peer.conn
        .writer()
        .write_all(b"secure")
        .expect("queue plaintext");
    let mut encrypted = Vec::new();
    while peer.conn.wants_write() {
        peer.conn.write_tls(&mut encrypted).expect("encode record");
    }
    peer.sock
        .write_all(&encrypted[..1])
        .expect("partial TLS record");
    let checks = AtomicUsize::new(0);
    assert_eq!(
        connections
            .read_with_cancellation(handle, 8, || checks.fetch_add(1, Ordering::Relaxed) >= 3),
        Err("Network read cancelled".to_string())
    );
    peer.sock
        .write_all(&encrypted[1..])
        .expect("finish TLS record");
    assert_eq!(
        connections
            .read_with_cancellation(handle, 8, || false)
            .expect("read after cancellation"),
        b"secure"
    );
}
