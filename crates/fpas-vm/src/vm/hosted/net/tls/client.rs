//! Verified TLS client setup for `Std.Net.ConnectTls`.

use std::net::TcpStream;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use rustls_platform_verifier::BuilderVerifierExt;

static CLIENT_CONFIG: OnceLock<Result<Arc<ClientConfig>, String>> = OnceLock::new();

/// Complete a TLS handshake using the operating system trust policy.
pub(in crate::vm::hosted::net) fn connect(
    socket: TcpStream,
    server_name: &str,
    timeout: Duration,
    is_cancelled: impl Fn() -> bool,
) -> Result<StreamOwned<ClientConnection, TcpStream>, String> {
    connect_with_config(socket, server_name, client_config()?, timeout, is_cancelled)
}

fn client_config() -> Result<Arc<ClientConfig>, String> {
    CLIENT_CONFIG.get_or_init(platform_config).clone()
}

fn platform_config() -> Result<Arc<ClientConfig>, String> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| format!("Could not configure TLS protocol versions: {error}"))?;
    let config = builder
        .with_platform_verifier()
        .map_err(|error| format!("Could not configure platform TLS verification: {error}"))?
        .with_no_client_auth();
    Ok(Arc::new(config))
}

fn connect_with_config(
    socket: TcpStream,
    server_name: &str,
    config: Arc<ClientConfig>,
    timeout: Duration,
    is_cancelled: impl Fn() -> bool,
) -> Result<StreamOwned<ClientConnection, TcpStream>, String> {
    let server_name = ServerName::try_from(server_name.to_owned())
        .map_err(|error| format!("Invalid TLS server name '{server_name}': {error}"))?;
    let connection = ClientConnection::new(config, server_name)
        .map_err(|error| format!("Could not create TLS client: {error}"))?;
    let mut stream = StreamOwned::new(connection, socket);
    super::handshake::complete(&mut stream.sock, timeout, is_cancelled, |socket| {
        stream.conn.complete_io(socket)?;
        Ok(!stream.conn.is_handshaking())
    })?;
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;

    use rcgen::{CertifiedKey, generate_simple_self_signed};
    use rustls::pki_types::PrivatePkcs8KeyDer;
    use rustls::{ClientConfig, RootCertStore, ServerConfig, ServerConnection, StreamOwned};

    use super::super::super::transport::Transport;
    use super::connect_with_config;

    struct TlsFixture {
        certificate: rustls::pki_types::CertificateDer<'static>,
        server: Arc<ServerConfig>,
    }

    impl TlsFixture {
        fn new(name: &str) -> Self {
            let CertifiedKey { cert, signing_key } =
                generate_simple_self_signed([name.to_string()]).expect("generate certificate");
            let certificate = cert.der().clone();
            let private_key = PrivatePkcs8KeyDer::from(signing_key.serialize_der()).into();
            let provider = Arc::new(rustls::crypto::ring::default_provider());
            let server = ServerConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()
                .expect("TLS versions")
                .with_no_client_auth()
                .with_single_cert(vec![certificate.clone()], private_key)
                .expect("server certificate");
            Self {
                certificate,
                server: Arc::new(server),
            }
        }

        fn client_config(&self) -> Arc<ClientConfig> {
            let mut roots = RootCertStore::empty();
            roots.add(self.certificate.clone()).expect("add test root");
            let provider = Arc::new(rustls::crypto::ring::default_provider());
            Arc::new(
                ClientConfig::builder_with_provider(provider)
                    .with_safe_default_protocol_versions()
                    .expect("TLS versions")
                    .with_root_certificates(roots)
                    .with_no_client_auth(),
            )
        }

        fn spawn_server(&self) -> (u16, std::thread::JoinHandle<()>) {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind TLS listener");
            let port = listener.local_addr().expect("TLS listener address").port();
            let config = Arc::clone(&self.server);
            let server = std::thread::spawn(move || {
                let (socket, _) = listener.accept().expect("accept TLS client");
                let connection = ServerConnection::new(config).expect("create TLS server");
                let mut stream = StreamOwned::new(connection, socket);
                let mut request = [0_u8; 4];
                if stream.read_exact(&mut request).is_ok() {
                    assert_eq!(&request, b"ping");
                    stream.write_all(b"pong").expect("write TLS response");
                }
            });
            (port, server)
        }
    }

    #[test]
    fn verified_tls_connection_round_trip_preserves_bytes() {
        let fixture = TlsFixture::new("localhost");
        let (port, server) = fixture.spawn_server();
        let socket = TcpStream::connect(("127.0.0.1", port)).expect("connect TLS socket");
        let mut transport = Transport::tls_client(
            connect_with_config(
                socket,
                "localhost",
                fixture.client_config(),
                std::time::Duration::from_secs(5),
                || false,
            )
            .expect("complete TLS handshake"),
        );

        transport.write_all(b"ping").expect("write TLS request");
        let mut response = [0_u8; 4];
        transport
            .read_exact(&mut response)
            .expect("read TLS response");

        assert_eq!(&response, b"pong");
        transport.shutdown().expect("close TLS transport");
        server.join().expect("join TLS server");
    }

    #[test]
    fn tls_handshake_rejects_wrong_hostname() {
        let fixture = TlsFixture::new("localhost");
        let (port, server) = fixture.spawn_server();
        let socket = TcpStream::connect(("127.0.0.1", port)).expect("connect TLS socket");

        let error = connect_with_config(
            socket,
            "wrong.example",
            fixture.client_config(),
            std::time::Duration::from_secs(5),
            || false,
        )
        .expect_err("hostname mismatch must fail");

        assert!(error.contains("certificate not valid for name"), "{error}");
        server.join().expect("join TLS server");
    }

    #[test]
    fn tls_handshake_rejects_untrusted_certificate() {
        let fixture = TlsFixture::new("localhost");
        let (port, server) = fixture.spawn_server();
        let socket = TcpStream::connect(("127.0.0.1", port)).expect("connect TLS socket");
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .expect("TLS versions")
            .with_root_certificates(RootCertStore::empty())
            .with_no_client_auth();

        let error = connect_with_config(
            socket,
            "localhost",
            Arc::new(config),
            std::time::Duration::from_secs(5),
            || false,
        )
        .expect_err("untrusted certificate must fail");

        assert!(error.contains("UnknownIssuer"), "{error}");
        server.join().expect("join TLS server");
    }
}
