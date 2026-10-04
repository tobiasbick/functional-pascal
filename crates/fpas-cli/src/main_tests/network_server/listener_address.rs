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
        &r#"program TlsListenerAddress;

uses Std.Net as Net;
uses Std.Test as Test;

function ExerciseListener(): Result of (boolean, string);
begin
  var Server: Net.Listener := try Net.ListenTls('127.0.0.1', 0, 'cert.pem', 'key.pem', 2000);
  var Address: Net.NetworkAddress := try Net.ListenerLocalAddress(Server);
  Test.AssertEquals('127.0.0.1', Address.Host);
  Test.AssertTrue(Address.Port > 0);
  Test.AssertTrue(Address.Port <= 65535);
  var Client: Net.Connection := try Net.Connect(Address.Host, Address.Port, 2000);
  var ClientClosed: boolean := try Net.Close(Client);
  var ServerClosed: boolean := try Net.CloseListener(Server);
  case Net.ListenerLocalAddress(Server) of
    when Result.Ok(_):
      panic('Closed TLS listener returned an address');
    when Result.Error(_):
      begin
        null;
      end;
  end case;

  return Result.Ok(true);
end function;

begin
  case ExerciseListener() of
    when Result.Ok(_):
      begin
        null;
      end;
    when Result.Error(const Message):
      panic(Message);
  end case;
end program;
"#
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
