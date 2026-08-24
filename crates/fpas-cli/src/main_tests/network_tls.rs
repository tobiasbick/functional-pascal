use std::io::Read;
use std::net::TcpListener;
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
    let server = std::thread::spawn(move || {
        let (socket, _) = listener.accept().expect("accept HTTPS client");
        let connection = ServerConnection::new(Arc::new(server_config)).expect("create TLS server");
        let mut stream = StreamOwned::new(connection, socket);
        let mut byte = [0_u8; 1];
        let _ = stream.read(&mut byte);
    });

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let cwd = create_temp_dir("https-untrusted-certificate");
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        &format!(
            r#"program HttpsUntrustedCertificate;

uses Std.Console, Std.Http, Std.Str;

begin
  case Send(Request.Get('https://localhost:{port}/')) of
    Ok(ResponseValue):
    begin
      panic('untrusted HTTPS server was accepted')
    end;
    Error(Message):
    begin
      if not Std.Str.Contains(Message, 'TLS handshake failed') then
      begin
        panic(Message)
      end
    end
  end;
  WriteLn('ok')
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
    server.join().expect("HTTPS fixture must finish");

    assert_eq!(exit, 0, "stderr: {stderr}");
    assert_eq!(stdout, "ok\n");
}
