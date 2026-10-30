use super::network_fixture::Listener as TcpListener;
use super::process::TempDir;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use rcgen::{CertifiedKey, generate_simple_self_signed};
use rustls::pki_types::PrivatePkcs8KeyDer;
use rustls::{ServerConfig, ServerConnection, StreamOwned};

use super::*;

#[test]
fn https_client_rejects_untrusted_server_certificate() {
    let CertifiedKey { cert, signing_key } =
        generate_simple_self_signed(["localhost".to_string()]).expect("generate certificate");
    let private_key = PrivatePkcs8KeyDer::from(signing_key.serialize_der()).into();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let server_config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("TLS versions")
        .with_no_client_auth()
        .with_single_cert(vec![cert.der().clone()], private_key)
        .expect("server certificate");
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind TLS fixture");
    let port = listener.local_addr().expect("TLS fixture address").port();
    let server = listener.spawn(move |listener| {
        let (socket, _) = listener.accept().expect("accept HTTPS client");
        let connection = ServerConnection::new(Arc::new(server_config)).expect("create TLS server");
        let mut stream = StreamOwned::new(connection, socket);
        let mut byte = [0_u8; 1];
        let error = stream
            .read(&mut byte)
            .expect_err("untrusted certificate must stop the handshake");
        assert!(
            matches!(
                error
                    .get_ref()
                    .and_then(|error| error.downcast_ref::<rustls::Error>()),
                Some(rustls::Error::AlertReceived(
                    rustls::AlertDescription::UnknownCA
                        | rustls::AlertDescription::CertificateUnknown
                        | rustls::AlertDescription::BadCertificate
                ))
            ),
            "client did not reject the presented certificate: {error}"
        );
    });

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let cwd = TempDir::new("https-untrusted-certificate").expect("fixture directory");
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        &format!(
            r#"program HttpsUntrustedCertificate;

uses Std.Console as Console, Std.Net.Http as Http, Std.Net.Sockets as Net;

begin
  case Http.Send(Http.Request.Get('https://localhost:{port}/')) of
    when Ok(const ResponseValue):
    begin
      panic('untrusted HTTPS server was accepted');
    end;
    when Error(const Message):
    begin
      if not (Message is Http.HttpError.Transport(Net.NetError.TlsFailure(_))) then
      begin
        panic(Message.Message());
      end; end if;
    end;
  end case;
  Console.WriteLn('ok');
end.
"#
        ),
    );

    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("run"),
            String::from("--std-lib"),
            root.join("lib").to_string_lossy().into_owned(),
            source.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    std::fs::remove_dir_all(&cwd).expect("temporary directory must be removed");
    assert_eq!(exit, 0, "stderr: {stderr}");
    server.join().expect("HTTPS fixture must finish");
    assert_eq!(stdout, "ok\n");
}
