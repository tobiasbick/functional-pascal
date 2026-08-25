use std::io::{Read, Write};
use std::sync::Arc;

use rcgen::{CertifiedKey, generate_simple_self_signed};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

use super::{connect_when_ready, finish_server, spawn_server, unused_port};

#[test]
fn fpas_https_server_serves_one_verified_request() {
    let port = unused_port();
    let CertifiedKey { cert, signing_key } =
        generate_simple_self_signed(["localhost".to_string()]).expect("generate certificate");
    let cwd = super::create_temp_dir("https-server");
    let certificate_path = cwd.join("certificate.pem");
    let private_key_path = cwd.join("private-key.pem");
    std::fs::write(&certificate_path, cert.pem()).expect("write certificate PEM");
    std::fs::write(&private_key_path, signing_key.serialize_pem()).expect("write private key PEM");
    let certificate_source = certificate_path.to_string_lossy().replace('\\', "/");
    let private_key_source = private_key_path.to_string_lossy().replace('\\', "/");
    let server = spawn_server(
        &cwd,
        format!(
            r#"program HttpsServer;

uses Std.Console, Std.Http, Std.Net, Std.Net.Utf8;

function Handle(RequestValue: ServerRequest): ServerResponse;
begin
  mutable var ResponseValue: ServerResponse := ServerResponse.Create(200, 'OK');
  ResponseValue.Body := Std.Net.Utf8.Encode('secure ' + RequestValue.Target);
  return ResponseValue
end;

begin
  case ListenTls('127.0.0.1', {port}, '{certificate_source}', '{private_key_source}', 2000) of
    Ok(ListenerValue):
    begin
      mutable var Options: ServerOptions := ServerOptions.Create();
      Options.MaxRequests := 1;
      case Serve(ListenerValue, Options, Handle) of
        Ok(_):
        begin
        end;
        Error(Message): panic(Message)
      end;
      case CloseListener(ListenerValue) of
        Ok(_): WriteLn('served https');
        Error(Message): panic(Message)
      end
    end;
    Error(Message): panic(Message)
  end
end.
"#
        ),
    );

    let mut roots = RootCertStore::empty();
    roots
        .add(cert.der().clone())
        .expect("trust test certificate");
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("TLS versions")
        .with_root_certificates(roots)
        .with_no_client_auth();
    let connection = ClientConnection::new(
        Arc::new(config),
        ServerName::try_from("localhost".to_string()).expect("server name"),
    )
    .expect("TLS client connection");
    let socket = connect_when_ready(port);
    let mut client = StreamOwned::new(connection, socket);
    client
        .write_all(b"GET /hello HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .expect("write HTTPS request");
    let mut response = Vec::new();
    client
        .read_to_end(&mut response)
        .expect("read HTTPS response");
    let (stdout, _) = finish_server(cwd, server);

    assert_eq!(stdout, "served https\n");
    assert_eq!(
        response,
        b"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 13\r\n\r\nsecure /hello"
    );
}
