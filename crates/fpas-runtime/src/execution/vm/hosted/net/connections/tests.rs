use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::NetworkConnections;

#[test]
fn shutdown_interrupts_outgoing_tls_handshake() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind silent peer");
    let address = listener.local_addr().expect("peer address");
    let connections = Arc::new(NetworkConnections::new());
    let client_connections = Arc::clone(&connections);
    let (send, receive) = std::sync::mpsc::channel();
    let client = std::thread::spawn(move || {
        let result = client_connections.connect_tls("127.0.0.1", i64::from(address.port()), 10_000);
        send.send(result).expect("report handshake result");
    });
    let (mut peer, _) = listener.accept().expect("accept TLS client");
    peer.set_read_timeout(Some(Duration::from_secs(5)))
        .expect("bound fixture read");
    let mut hello = [0; 1];
    peer.read_exact(&mut hello)
        .expect("client entered TLS handshake");
    connections.shutdown();
    let result = receive.recv_timeout(Duration::from_secs(2));
    // Release the worker independently even when cancellation is broken.
    drop(peer.shutdown(std::net::Shutdown::Both));
    client.join().expect("join TLS client");
    assert!(
        result
            .expect("shutdown must interrupt pending handshake")
            .is_err()
    );
    assert!(connections.connections.lock().expect("registry").is_empty());
}

#[test]
fn tcp_connection_round_trip_preserves_bytes() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local listener");
    let address = listener.local_addr().expect("listener address");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept client");
        let mut request = [0_u8; 4];
        stream.read_exact(&mut request).expect("read request");
        assert_eq!(&request, b"ping");
        stream.write_all(b"pong").expect("write response");
    });

    let connections = NetworkConnections::new();
    let handle = connections
        .connect_tcp("127.0.0.1", i64::from(address.port()), 1_000)
        .expect("connect client");
    assert_eq!(connections.write(handle, b"ping").expect("write"), 4);
    assert_eq!(connections.read(handle, 16).expect("read"), b"pong");
    connections.close(handle).expect("close");
    server.join().expect("join server");
}

#[test]
fn closed_connection_cannot_be_reused() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local listener");
    let address = listener.local_addr().expect("listener address");
    let server = std::thread::spawn(move || listener.accept().expect("accept client"));

    let connections = NetworkConnections::new();
    let handle = connections
        .connect_tcp("127.0.0.1", i64::from(address.port()), 1_000)
        .expect("connect client");
    connections.close(handle).expect("close");
    assert!(connections.read(handle, 1).is_err());
    drop(server.join().expect("join server"));
}

#[test]
fn close_interrupts_a_blocked_read() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local listener");
    let address = listener.local_addr().expect("listener address");
    let connections = Arc::new(NetworkConnections::new());
    let handle = connections
        .connect_tcp("127.0.0.1", i64::from(address.port()), 1_000)
        .expect("connect client");
    let (_server, _) = listener.accept().expect("accept client");
    connections.set_timeout(handle, 500).expect("set timeout");

    let reader_connections = Arc::clone(&connections);
    let reader = std::thread::spawn(move || reader_connections.read(handle, 1));
    std::thread::sleep(Duration::from_millis(50));

    let started = Instant::now();
    connections.close(handle).expect("close");
    let close_elapsed = started.elapsed();
    let _ = reader.join().expect("join reader");

    assert!(
        close_elapsed < Duration::from_millis(200),
        "close waited {close_elapsed:?} for the blocked read"
    );
}
