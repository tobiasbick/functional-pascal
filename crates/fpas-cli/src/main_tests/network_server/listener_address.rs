//! Public TLS listener address and lifetime regression.

use super::*;

#[test]
fn tls_listener_reports_an_os_assigned_address() {
    let cwd = create_temp_dir("tls-listener-address");
    let rcgen::CertifiedKey { cert, signing_key } =
        rcgen::generate_simple_self_signed(["localhost".to_string()]).expect("certificate");
    write_text(&cwd.join("cert.pem"), &cert.pem());
    write_text(&cwd.join("key.pem"), &signing_key.serialize_pem());
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        &"program TlsListenerAddress;
uses Std.Net, Std.Test;
function ExerciseListener(): result of boolean, string;
begin
  var Server: Listener := try ListenTls('127.0.0.1', 0, 'cert.pem', 'key.pem', 2000);
  var Address: NetworkAddress := try ListenerLocalAddress(Server);
  AssertEquals('127.0.0.1', Address.Host);
  AssertTrue(Address.Port > 0);
  AssertTrue(Address.Port <= 65535);
  var Client: Connection := try Connect(Address.Host, Address.Port, 2000);
  var ClientClosed: boolean := try Close(Client);
  var ServerClosed: boolean := try CloseListener(Server);
  case ListenerLocalAddress(Server) of
    Ok(_): panic('Closed TLS listener returned an address');
    Error(_): begin end
  end;
  return Ok(true)
end;
begin
  case ExerciseListener() of
    Ok(_): begin end;
    Error(Message): panic(Message)
  end
end."
            .replace(
                "cert.pem",
                &cwd.join("cert.pem")
                    .to_string_lossy()
                    .replace('\\', "/")
                    .replace('\'', "''"),
            )
            .replace(
                "key.pem",
                &cwd.join("key.pem")
                    .to_string_lossy()
                    .replace('\\', "/")
                    .replace('\'', "''"),
            ),
    );

    let (code, _, stderr) = support::run_cli_and_capture_output(&source, &cwd);
    fs::remove_dir_all(&cwd).expect("remove fixture");
    assert_eq!(code, 0, "{stderr}");
}
