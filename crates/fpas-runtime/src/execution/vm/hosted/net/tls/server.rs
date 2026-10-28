//! Certificate-configured TLS server setup for `Std.Net.Sockets.ListenTls`.

use crate::execution::vm::hosted::net::failure::Failure;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};

const MAX_HANDSHAKE_TIMEOUT_MILLIS: u64 = 300_000;

/// TLS configuration retained by one listener.
#[derive(Clone)]
pub(in crate::execution::vm::hosted::net) struct TlsServer {
    config: Arc<ServerConfig>,
    handshake_timeout: Duration,
}

impl TlsServer {
    /// Load one PEM certificate chain and private key.
    pub(in crate::execution::vm::hosted::net) fn from_pem_files(
        certificate_path: &str,
        private_key_path: &str,
        handshake_timeout_millis: i64,
    ) -> Result<Self, Failure> {
        let handshake_timeout = handshake_timeout(handshake_timeout_millis)?;
        let certificates = CertificateDer::pem_file_iter(Path::new(certificate_path))
            .map_err(|error| {
                Failure::pem(
                    &error,
                    format!("Could not open TLS certificate chain: {error}"),
                )
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                Failure::pem(
                    &error,
                    format!("Could not parse TLS certificate chain: {error}"),
                )
            })?;
        if certificates.is_empty() {
            return Err(Failure::Tls(
                "TLS certificate chain contains no certificates".to_string(),
            ));
        }
        let private_key =
            PrivateKeyDer::from_pem_file(Path::new(private_key_path)).map_err(|error| {
                Failure::pem(&error, format!("Could not load TLS private key: {error}"))
            })?;
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|error| {
                Failure::Tls(format!(
                    "Could not configure TLS protocol versions: {error}"
                ))
            })?
            .with_no_client_auth()
            .with_single_cert(certificates, private_key)
            .map_err(|error| {
                Failure::Tls(format!(
                    "Could not configure TLS certificate and private key: {error}"
                ))
            })?;
        Ok(Self {
            config: Arc::new(config),
            handshake_timeout,
        })
    }

    /// Complete one server-side TLS handshake.
    pub(in crate::execution::vm::hosted::net) fn accept(
        &self,
        mut socket: TcpStream,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<StreamOwned<ServerConnection, TcpStream>, Failure> {
        let mut connection = ServerConnection::new(Arc::clone(&self.config)).map_err(|error| {
            Failure::Tls(format!("Could not create TLS server connection: {error}"))
        })?;
        super::handshake::complete(
            &mut socket,
            self.handshake_timeout,
            is_cancelled,
            |socket| {
                connection.complete_io(socket)?;
                Ok(!connection.is_handshaking())
            },
        )?;
        Ok(StreamOwned::new(connection, socket))
    }
}

fn handshake_timeout(millis: i64) -> Result<Duration, Failure> {
    let millis = u64::try_from(millis)
        .ok()
        .filter(|millis| (1..=MAX_HANDSHAKE_TIMEOUT_MILLIS).contains(millis))
        .ok_or_else(|| {
            Failure::argument("HandshakeTimeoutMillis", format!("TLS handshake timeout must be in 1..={MAX_HANDSHAKE_TIMEOUT_MILLIS} ms, got {millis}"))
        })?;
    Ok(Duration::from_millis(millis))
}

#[cfg(test)]
mod tests {
    use super::Failure;
    use super::TlsServer;

    #[test]
    fn tls_listener_rejects_invalid_timeout_before_reading_files() {
        let error = TlsServer::from_pem_files("missing-cert", "missing-key", 0)
            .err()
            .expect("zero handshake timeout must fail");

        assert!(
            matches!(
                error,
                Failure::InvalidArgument {
                    name: "HandshakeTimeoutMillis",
                    ..
                }
            ),
            "{error}"
        );
    }
}
