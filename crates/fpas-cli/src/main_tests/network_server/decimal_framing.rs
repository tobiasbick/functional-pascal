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
uses Std.Http as Http; uses Std.Net as Net; uses Std.Net.Utf8 as Utf8; uses Std.Results as Results; uses Std.Str as Str;
function Require of (T)(Outcome: Result of (T, string)): T;
begin
  case Outcome of
    when Result.Ok(const Value): return Value;
    when Result.Error(const Message): panic(Message);
  end case;
end function;
begin
  const ListenerValue: Net.Listener := Require(Net.Listen('127.0.0.1', {port}));
  for I: integer := 1 to {} do
  begin
    const ConnectionValue: Net.Connection := Require(Net.Accept(ListenerValue));
    discard Results.Unwrap(Net.SetTimeout(ConnectionValue, 2000));
     var Text: string := 'accepted';
    case Http.ReadRequest(ConnectionValue, 4096, 16) of
      when Result.Ok(_): begin null; end;
      when Result.Error(const Message): begin
        if I = {} then
        begin if not Str.Contains(Message, 'MaxBodyBytes') then panic(Message); end if; end;
        else begin if not Str.Contains(Message, 'Content-Length') then panic(Message); end if; end; end if;
        Text := 'rejected';
      end;
    end case;
     var ResponseValue: Http.ServerResponse := Http.ServerResponseCreate(200, 'OK');
    ResponseValue.Body := Utf8.Encode(Text);
    discard Results.Unwrap(Http.WriteResponse(ConnectionValue, ResponseValue));
    discard Results.Unwrap(Net.Close(ConnectionValue));
  end; end for;
  discard Results.Unwrap(Net.CloseListener(ListenerValue));
end program;
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
