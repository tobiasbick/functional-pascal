//! Exact redirect targets and decimal framing through loopback HTTP.
use super::*;
fn accept(listener: &TcpListener) -> TcpStream {
    listener.accept().expect("accept HTTP fixture").0
}

#[test]
fn source_review_redirects_preserve_exact_request_paths() {
    let cases = [
        ("/base/start", "/target/", "/target/"),
        ("/base/start", "/a//b", "/a//b"),
        ("/base/start", "/a//../b", "/a/b"),
        ("/base/start", "./", "/base/"),
        ("/base/start", "../", "/"),
        ("/base/start", "../other//", "/other//"),
        ("/base/start", "?new=1", "/base/start?new=1"),
        ("/base/start", "#fragment", "/base/start"),
        ("/base/start", "./item/../?q=/./", "/base/?q=/./"),
        ("/base/start", "/%2e/", "/%2e/"),
        ("/a/../b", "?new=1", "/a/../b?new=1"),
        ("/a/../b", "#fragment", "/a/../b"),
    ];
    let listener = TcpListener::bind("127.0.0.1:0").expect("fixture");
    let port = listener.local_addr().expect("address").port();
    let paths = cases
        .iter()
        .map(|(base, _, _)| format!("'{base}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let server = listener.spawn(move |listener| {
        for (base, location, expected) in cases {
            let mut first = accept(&listener);
            assert!(read_request(&mut first).starts_with(&format!("GET {base} HTTP/1.1\r\n")));
            write!(first, "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").expect("redirect");
            drop(first);
            let mut next = accept(&listener);
            assert!(
                read_request(&mut next).starts_with(&format!("GET {expected} HTTP/1.1\r\n")),
                "location {location}"
            );
            next.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .expect("response");
        }
    });
    let (exit, _, stderr) = run_program(
        "source-review-redirect",
        &format!(
            r#"program RedirectPaths;
uses Std.Net.Http as Http;
begin
  for Base: string in [{paths}] do
  begin
    case Http.Send(Http.Request.Get('http://127.0.0.1:{port}' + Base)) of
      when Ok(_): begin end;
      when Error(const Message): panic(Message.Message());
    end case;
  end; end for;
end.
"#
        ),
    );
    assert_eq!(exit, 0, "{stderr}");
    server.join().expect("fixture completed");
}

#[test]
fn source_review_client_rejects_non_decimal_lengths() {
    let values = [
        "+2",
        "-2",
        "1_0",
        "$2",
        "0x2",
        "2 0",
        "9223372036854775808",
        "",
    ];
    let listener = TcpListener::bind("127.0.0.1:0").expect("fixture");
    let port = listener.local_addr().expect("address").port();
    let server = listener.spawn(move |listener| {
        for value in values {
            let mut stream = accept(&listener);
            read_request(&mut stream);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {value}\r\nConnection: close\r\n\r\n0123456789"
            );
            match stream.write_all(response.as_bytes()) {
                Ok(()) => {}
                // Rejecting the malformed header can close the peer before the full reply is sent.
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
                    ) => {}
                Err(error) => panic!("response: {error}"),
            }
        }
    });
    let (exit, _, stderr) = run_program(
        "source-review-lengths",
        &format!(
            r#"program DecimalLengths;
uses Std.Net.Http as Http;
begin
  for I: integer := 1 to {} do
  begin
    case Http.Send(Http.Request.Get('http://127.0.0.1:{port}/')) of
      when Ok(_): panic('invalid Content-Length accepted');
      when Error(const Message): begin
        if not (Message is Http.HttpError.InvalidMessage(_)) then panic(Message.Message()); end if;
      end;
    end case;
  end; end for;
end.
"#,
            values.len()
        ),
    );
    assert_eq!(exit, 0, "{stderr}");
    server.join().expect("fixture completed");
}
