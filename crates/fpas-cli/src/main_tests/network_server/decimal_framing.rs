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
            "program DecimalServer;\nuses Std.Http as Http; uses Std.Net as Net; uses Std.Net.Utf8 as Utf8; uses Std.Results as Results; uses Std.Str as Str;\nbegin\n  var ListenerValue: Net.Listener := Results.Unwrap(Net.Listen('127.0.0.1', {port}));\n  for I: integer := 1 to {} do\n  begin\n    var ConnectionValue: Net.Connection := Results.Unwrap(Net.Accept(ListenerValue));\n    Results.Unwrap(Net.SetTimeout(ConnectionValue, 2000));\n    mutable var Text: string := 'accepted';\n    case Http.ReadRequest(ConnectionValue, 4096, 16) of\n      when Ok(_): begin null; end;\n      when Error(Message): begin\n        if I = {} then\n        begin if not Str.Contains(Message, 'MaxBodyBytes') then panic(Message); end if; end;\n        else begin if not Str.Contains(Message, 'Content-Length') then panic(Message); end if; end; end if;\n        Text := 'rejected';\n      end;\n    end case;\n    mutable var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(200, 'OK');\n    ResponseValue.Body := Utf8.Encode(Text);\n    Results.Unwrap(Http.WriteResponse(ConnectionValue, ResponseValue));\n    Results.Unwrap(Net.Close(ConnectionValue));\n  end; end for;\n  Results.Unwrap(Net.CloseListener(ListenerValue));\nend program;\n",
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
