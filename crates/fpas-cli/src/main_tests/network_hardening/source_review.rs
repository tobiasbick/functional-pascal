//! Exact redirect targets and decimal framing through loopback HTTP.
use super::*;
use std::time::{Duration, Instant};

fn accept(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).expect("nonblocking fixture");
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                // Accepted sockets can inherit the listener's nonblocking mode (Windows), which
                // turns a not-yet-arrived request into a spurious `WouldBlock` read failure.
                stream
                    .set_nonblocking(false)
                    .expect("blocking fixture connection");
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .expect("read deadline");
                return stream;
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(error) => panic!("accept HTTP fixture: {error}"),
        }
    }
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
    let server = std::thread::spawn(move || {
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
uses Std.Http;
begin
  for Base: string in [{paths}] do
  begin
    case Send(Request.Get('http://127.0.0.1:{port}' + Base)) of
      Ok(_): begin end;
      Error(Message): panic(Message)
    end
  end
end.
"#
        ),
    );
    server.join().expect("fixture completed");
    assert_eq!(exit, 0, "{stderr}");
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
    let server = std::thread::spawn(move || {
        for value in values {
            let mut stream = accept(&listener);
            read_request(&mut stream);
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {value}\r\nConnection: close\r\n\r\n0123456789"
            )
            .expect("response");
        }
    });
    let (exit, _, stderr) = run_program(
        "source-review-lengths",
        &format!(
            r#"program DecimalLengths;
uses Std.Http, Std.Str;
begin
  for I: integer := 1 to {} do
  begin
    case Send(Request.Get('http://127.0.0.1:{port}/')) of
      Ok(_): panic('invalid Content-Length accepted');
      Error(Message): begin
        if not Std.Str.Contains(Message, 'Content-Length') then panic(Message)
      end
    end
  end
end.
"#,
            values.len()
        ),
    );
    server.join().expect("fixture completed");
    assert_eq!(exit, 0, "{stderr}");
}
