//! Strict decimal request lengths through the public HTTP server.
use super::*;

#[test]
fn source_review_server_checks_decimal_lengths_and_boundaries() {
    let cases = [
        ("+2", false),
        ("-2", false),
        ("1_0", false),
        ("$2", false),
        ("0x2", false),
        ("9223372036854775808", false),
        ("", false),
        ("0", true),
        ("0002", true),
        ("9223372036854775807", false),
    ];
    let (cwd, server) = start_server(
        "source-review-server-decimal",
        format!(
            r#"program DecimalServer;
uses Std.Console as Console, Std.Net.Http as Http, Std.Net.Sockets as Net, Std.Text.Utf8 as Utf8;
begin
  const ListenerValue: Net.Listener := Net.Listen('127.0.0.1', 0).Unwrap();
  Console.WriteLn('FPAS_SERVER_FIXTURE_PORT=', Net.ListenerLocalAddress(ListenerValue).Unwrap().Port);
  for I: integer := 1 to {} do
  begin
    const ConnectionValue: Net.Connection := Net.Accept(ListenerValue).Unwrap();
    discard Net.SetTimeout(ConnectionValue, 2000).Unwrap();
    var Text: string := 'accepted';
    case Http.ReadRequest(ConnectionValue, 4096, 16) of
      when Ok(_): begin end;
      when Error(const Message): begin
        if I = {} then
        begin if not (Message is Http.HttpError.LimitExceeded('Std.Net.Http.Server.ReadRequest', _)) then panic(Message.Message()); end if; end;
        else begin if not (Message is Http.HttpError.InvalidMessage(_)) then panic(Message.Message()); end if; end; end if;
        Text := 'rejected';
      end;
    end case;
    var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(200, 'OK');
    ResponseValue.Body := Utf8.Encode(Text).Unwrap();
    discard Http.WriteResponse(ConnectionValue, ResponseValue).Unwrap();
    discard Net.Close(ConnectionValue).Unwrap();
  end; end for;
  discard Net.CloseListener(ListenerValue).Unwrap();
end.
"#,
            cases.len(),
            cases.len()
        ),
    );
    for (value, valid) in cases {
        let request =
            format!("POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: {value}\r\n\r\nok");
        let response = exchange(server.port(), &[request.as_bytes()]);
        assert!(
            response.ends_with(if valid { b"accepted" } else { b"rejected" }),
            "length {value}: {}",
            String::from_utf8_lossy(&response)
        );
    }
    finish_server(cwd, server);
}
