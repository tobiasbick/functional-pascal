//! Real TLS backpressure preserves accepted bytes and releases detached close state.

use super::super::test_connections::tls_pair;
use super::*;
use std::io::Read;

fn blocked() -> (
    NetworkConnections,
    u64,
    rustls::StreamOwned<rustls::ServerConnection, std::net::TcpStream>,
    usize,
) {
    let (client, server) = tls_pair();
    socket2::SockRef::from(&client.sock)
        .set_send_buffer_size(4096)
        .unwrap();
    socket2::SockRef::from(&server.sock)
        .set_recv_buffer_size(4096)
        .unwrap();
    server
        .sock
        .set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    let mut transport = Transport::tls_client(client);
    transport
        .set_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    transport.set_nonblocking(true).unwrap();
    let mut accepted = 0;
    let mut pending = false;
    for _ in 0..512 {
        match transport.write(&[42; 8192]) {
            Ok(count) => accepted += count,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                pending = true;
                break;
            }
            Err(error) => panic!("TLS fixture write: {error}"),
        }
        match transport.flush() {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                pending = true;
                break;
            }
            Err(error) => panic!("TLS fixture flush: {error}"),
        }
    }
    assert!(
        pending && accepted > 0,
        "fixture must have pending application bytes"
    );
    let connections = NetworkConnections::new();
    let handle = connections.insert_accepted(transport).unwrap();
    (connections, handle, server, accepted)
}

#[test]
fn tls_close_yields_to_its_peer_and_preserves_every_accepted_byte() {
    let (connections, handle, mut peer, accepted) = blocked();
    let alias = connections.connection(handle).unwrap();
    let mut closing = connections.begin_close(handle).unwrap();
    assert!(closing.poll().is_pending());
    assert!(matches!(
        connections.connection(handle),
        Err(Failure::Closed)
    ));
    assert!(matches!(
        *alias.transport.lock().unwrap(),
        Transport::Closed
    ));
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        peer.read_to_end(&mut bytes).unwrap();
        bytes
    });
    closing.finish().unwrap();
    let bytes = reader.join().unwrap();
    assert_eq!(bytes.len(), accepted);
    assert!(bytes.iter().all(|&byte| byte == 42));
}

#[test]
fn abandoned_tls_close_interrupts_the_socket_even_with_a_retained_alias() {
    let (connections, handle, mut peer, _) = blocked();
    let _alias = connections.connection(handle).unwrap();
    let mut closing = connections.begin_close(handle).unwrap();
    assert!(closing.poll().is_pending());
    drop(closing);
    peer.sock
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    match peer.sock.read_to_end(&mut Vec::new()) {
        Ok(_) => {}
        Err(error) => assert!(matches!(
            error.kind(),
            io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted
        )),
    }
}
