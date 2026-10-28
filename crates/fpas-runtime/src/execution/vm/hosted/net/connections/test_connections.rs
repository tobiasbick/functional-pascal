//! Local connected socket fixtures for network cancellation tests.

use super::NetworkConnections;
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{PrivatePkcs8KeyDer, ServerName};
use rustls::{
    ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection, StreamOwned,
};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

/// Establish a locally trusted TLS pair with bounded handshake reads.
pub(super) fn tls_pair() -> (
    StreamOwned<ClientConnection, TcpStream>,
    StreamOwned<ServerConnection, TcpStream>,
) {
    let certificate =
        generate_simple_self_signed(vec!["localhost".to_string()]).expect("certificate");
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let server_config = ServerConfig::builder_with_provider(Arc::clone(&provider))
        .with_safe_default_protocol_versions()
        .expect("versions")
        .with_no_client_auth()
        .with_single_cert(
            vec![certificate.cert.der().clone()],
            PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der()).into(),
        )
        .expect("server config");
    let mut roots = RootCertStore::empty();
    roots
        .add(certificate.cert.der().clone())
        .expect("trust fixture");
    let client_config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("versions")
        .with_root_certificates(roots)
        .with_no_client_auth();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let socket = TcpStream::connect(listener.local_addr().expect("address")).expect("connect");
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout");
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("accept");
        socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("timeout");
        let mut connection = ServerConnection::new(Arc::new(server_config)).expect("server");
        while connection.is_handshaking() {
            connection.complete_io(&mut socket).expect("handshake");
        }
        StreamOwned::new(connection, socket)
    });
    let mut client = StreamOwned::new(
        ClientConnection::new(
            Arc::new(client_config),
            ServerName::try_from("localhost").expect("name"),
        )
        .expect("client"),
        socket,
    );
    while client.conn.is_handshaking() {
        client
            .conn
            .complete_io(&mut client.sock)
            .expect("handshake");
    }
    (client, server.join().expect("server"))
}

/// Establish a plain TCP pair without sending application data.
pub(super) fn tcp_pair() -> (NetworkConnections, u64, TcpStream) {
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
    peer.set_read_timeout(Some(Duration::from_secs(2)))
        .expect("fixture timeout");
    (connections, handle, peer)
}
