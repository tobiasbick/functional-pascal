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
        &"program TlsListenerAddress;\nuses Std.Net as Net, Std.Test as Test;\nfunction ExerciseListener(): result of (boolean, Net.NetError);\nbegin\n  const Server: Net.Listener := try Net.ListenTls('127.0.0.1', 0, 'cert.pem', 'key.pem', 2000);\n  const Address: Net.NetworkAddress := try Net.ListenerLocalAddress(Server);\n  Test.AssertEquals('127.0.0.1', Address.Host);\n  Test.AssertTrue(Address.Port > 0);\n  Test.AssertTrue(Address.Port <= 65535);\n  const Client: Net.Connection := try Net.Connect(Address.Host, Address.Port, 2000);\n  const ClientClosed: boolean := try Net.Close(Client);\n  const ServerClosed: boolean := try Net.CloseListener(Server);\n  case Net.ListenerLocalAddress(Server) of\n    when Ok(_): panic('Closed TLS listener returned an address');\n    when Error(_): begin end;\n  end case;\n  return Ok(true);\nend function;\nbegin\n  case ExerciseListener() of\n    when Ok(_): begin end;\n    when Error(const Message): panic(Message.Message());\n  end case;\nend."
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
