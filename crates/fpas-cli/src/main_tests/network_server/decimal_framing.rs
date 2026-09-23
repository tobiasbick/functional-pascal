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
    let port = unused_port();
    let (cwd, server) = start_server(
        "source-review-server-decimal",
        format!(
            r#"program DecimalServer;
uses Std.Http, Std.Net, Std.Net.Utf8, Std.Results, Std.Str;
begin
  var ListenerValue: Listener := Unwrap(Listen('127.0.0.1', {port}));
  for I: integer := 1 to {} do
  begin
    var ConnectionValue: Connection := Unwrap(Accept(ListenerValue));
    Unwrap(SetTimeout(ConnectionValue, 2000));
    mutable var Text: string := 'accepted';
    case ReadRequest(ConnectionValue, 4096, 16) of
      Ok(_): begin end;
      Error(Message): begin
        if I = {} then
        begin if not Std.Str.Contains(Message, 'MaxBodyBytes') then panic(Message) end
        else begin if not Std.Str.Contains(Message, 'Content-Length') then panic(Message) end;
        Text := 'rejected'
      end
    end;
    mutable var ResponseValue: ServerResponse := ServerResponse.Create(200, 'OK');
    ResponseValue.Body := Std.Net.Utf8.Encode(Text);
    Unwrap(WriteResponse(ConnectionValue, ResponseValue));
    Unwrap(Close(ConnectionValue))
  end;
  Unwrap(CloseListener(ListenerValue))
end.
"#,
            cases.len(),
            cases.len()
        ),
    );
    for (value, valid) in cases {
        let request =
            format!("POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: {value}\r\n\r\nok");
        let response = exchange(port, &[request.as_bytes()]);
        assert!(
            response.ends_with(if valid { b"accepted" } else { b"rejected" }),
            "length {value}: {}",
            String::from_utf8_lossy(&response)
        );
    }
    finish_server(cwd, server);
}
