use super::network_fixture::{Listener as TcpListener, Stream as TcpStream};
use super::process::TempDir;
use std::io::{Read, Write};
use std::path::Path;

use super::*;

mod source_review;

fn read_request(stream: &mut TcpStream) -> String {
    let mut request = Vec::new();
    loop {
        let mut chunk = [0_u8; 2048];
        let count = stream.read(&mut chunk).expect("read HTTP request");
        assert!(count > 0, "request ended before its body");
        request.extend_from_slice(&chunk[..count]);
        let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .and_then(|value| value.parse::<usize>().ok())
            .expect("Content-Length header");
        if request.len() >= header_end + 4 + content_length {
            return String::from_utf8(request).expect("UTF-8 HTTP request");
        }
    }
}

fn run_program(name: &str, source_text: &str) -> (i32, String, String) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let cwd = TempDir::new(name).expect("fixture directory");
    let source = cwd.join("main.fpas");
    write_text(&source, source_text);
    let output = support::run_cli_args_and_capture_output(
        &[
            String::from("run"),
            String::from("--std-lib"),
            root.join("lib").to_string_lossy().into_owned(),
            source.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    std::fs::remove_dir_all(&cwd).expect("temporary directory must be removed");
    output
}

#[test]
fn http_client_skips_informational_response_and_follows_relative_redirect() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind redirect fixture");
    let port = listener
        .local_addr()
        .expect("redirect fixture address")
        .port();
    let server = listener.spawn(move |listener| {
        let (mut first, _) = listener.accept().expect("accept initial request");
        assert!(read_request(&mut first).starts_with("GET /start HTTP/1.1\r\n"));
        first
            .write_all(
                b"HTTP/1.1 100 Continue\r\nX-Interim: yes\r\n\r\nHTTP/1.1 302 Found\r\nLocation: ./final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("write redirect response");

        let (mut second, _) = listener.accept().expect("accept redirected request");
        assert!(read_request(&mut second).starts_with("GET /final HTTP/1.1\r\n"));
        second
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
            .expect("write final response");
    });

    let (exit, stdout, stderr) = run_program(
        "http-relative-redirect",
        &format!(
            r#"program HttpRelativeRedirect;

uses Std.Console as Console, Std.Net.Http as Http;

begin
  case Http.Send(Http.Request.Get('http://127.0.0.1:{port}/start')) of
    when Ok(const ResponseValue):
    begin
      case Http.BodyText(ResponseValue) of
        when Ok(const Text): Console.WriteLn(Text);
        when Error(const Message): panic(Message.Message());
      end case;
    end;
    when Error(const Message): panic(Message.Message());
  end case;
end.
"#
        ),
    );
    assert_eq!(exit, 0, "stderr: {stderr}");
    server.join().expect("redirect fixture must finish");
    assert_eq!(stdout, "ok\n");
}

#[test]
fn cross_origin_303_redirect_changes_post_to_get_and_strips_credentials() {
    let redirect_listener = TcpListener::bind("127.0.0.1:0").expect("bind redirect origin");
    let redirect_port = redirect_listener
        .local_addr()
        .expect("redirect origin address")
        .port();
    let target_listener = TcpListener::bind("127.0.0.1:0").expect("bind target origin");
    let target_port = target_listener
        .local_addr()
        .expect("target origin address")
        .port();
    let redirect_server = redirect_listener.spawn(move |redirect_listener| {
        let (mut stream, _) = redirect_listener.accept().expect("accept POST request");
        let request = read_request(&mut stream);
        assert!(request.starts_with("POST /submit HTTP/1.1\r\n"));
        assert!(request.contains("Authorization: Bearer secret\r\n"));
        for header in [
            "content-type",
            "content-encoding",
            "content-language",
            "content-location",
        ] {
            assert!(request.to_ascii_lowercase().contains(&format!("{header}:")));
        }
        write!(
            stream,
            "HTTP/1.1 303 See Other\r\nLocation: http://127.0.0.1:{target_port}/result\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
        .expect("write cross-origin redirect");
    });
    let target_server = target_listener.spawn(move |target_listener| {
        let (mut stream, _) = target_listener.accept().expect("accept redirected GET");
        let request = read_request(&mut stream);
        assert!(request.starts_with("GET /result HTTP/1.1\r\n"));
        assert!(!request.contains("Authorization:"));
        assert!(!request.contains("Cookie:"));
        assert!(!request.contains("Content-Type:"));
        for header in ["content-encoding", "content-language", "content-location"] {
            assert!(!request.to_ascii_lowercase().contains(&format!("{header}:")));
        }
        assert!(request.contains("Content-Length: 0\r\n"));
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
            .expect("write redirected response");
    });

    let (exit, stdout, stderr) = run_program(
        "http-cross-origin-redirect",
        &format!(
            r#"program HttpCrossOriginRedirect;

uses Std.Console as Console, Std.Net.Http as Http, Std.Text.Utf8 as Utf8;

begin
  var RequestValue: Http.Request := Http.Request.Post('http://127.0.0.1:{redirect_port}/submit');
  RequestValue.Headers := [
    Http.Header.Create('Authorization', 'Bearer secret'),
    Http.Header.Create('Cookie', 'session=secret'),
    Http.Header.Create('Content-Type', 'text/plain'),
    Http.Header.Create('Content-Encoding', 'identity'),
    Http.Header.Create('content-language', 'en'),
    Http.Header.Create('Content-Location', '/submit')
  ];
  RequestValue.Body := Utf8.Encode('payload');
  case Http.Send(RequestValue) of
    when Ok(const ResponseValue): Console.WriteLn(ResponseValue.StatusCode);
    when Error(const Message): panic(Message.Message());
  end case;
end.
"#
        ),
    );
    assert_eq!(exit, 0, "stderr: {stderr}");
    redirect_server.join().expect("redirect origin must finish");
    target_server.join().expect("target origin must finish");
    assert_eq!(stdout, "200\n");
}

#[test]
fn http_client_enforces_header_and_redirect_limits() {
    let header_listener = TcpListener::bind("127.0.0.1:0").expect("bind header limit fixture");
    let header_port = header_listener
        .local_addr()
        .expect("header limit address")
        .port();
    let header_server = header_listener.spawn(move |header_listener| {
        let (mut stream, _) = header_listener
            .accept()
            .expect("accept header limit request");
        read_request(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nX-Oversized: 012345678901234567890123456789012345678901234567890123456789\r\n\r\n")
            .expect("write oversized head");
    });

    let redirect_listener = TcpListener::bind("127.0.0.1:0").expect("bind redirect limit fixture");
    let redirect_port = redirect_listener
        .local_addr()
        .expect("redirect limit address")
        .port();
    let redirect_server = redirect_listener.spawn(move |redirect_listener| {
        let (mut stream, _) = redirect_listener
            .accept()
            .expect("accept redirect limit request");
        read_request(&mut stream);
        stream
            .write_all(b"HTTP/1.1 302 Found\r\nLocation: /again\r\nContent-Length: 0\r\n\r\n")
            .expect("write prohibited redirect");
    });

    let (exit, stdout, stderr) = run_program(
        "http-client-limits",
        &format!(
            r#"program HttpClientLimits;

uses Std.Console as Console, Std.Net.Http as Http;

begin
  var HeaderRequest: Http.Request := Http.Request.Get('http://127.0.0.1:{header_port}/');
  HeaderRequest.MaxHeaderBytes := 48;
  case Http.Send(HeaderRequest) of
    when Ok(_): panic('oversized response head was accepted');
    when Error(const Message):
    begin
      if not (Message is Http.HttpError.LimitExceeded('Std.Net.Http.Head.ReadFinal', 48)) then panic(Message.Message()); end if;
    end;
  end case;
  var RedirectRequest: Http.Request := Http.Request.Get('http://127.0.0.1:{redirect_port}/');
  RedirectRequest.MaxRedirects := 0;
  case Http.Send(RedirectRequest) of
    when Ok(_): panic('redirect limit was ignored');
    when Error(const Message):
    begin
      if not (Message is Http.HttpError.RedirectFailure(_)) then panic(Message.Message()); end if;
    end;
  end case;
  Console.WriteLn('ok');
end.
"#
        ),
    );
    assert_eq!(exit, 0, "stderr: {stderr}");
    header_server
        .join()
        .expect("header limit fixture must finish");
    redirect_server
        .join()
        .expect("redirect limit fixture must finish");
    assert_eq!(stdout, "ok\n");
}

#[test]
fn http_client_rejects_ambiguous_framing_and_excess_interim_responses() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind hostile response fixture");
    let port = listener
        .local_addr()
        .expect("hostile response address")
        .port();
    let server = listener.spawn(move |listener| {
        let (mut framing, _) = listener.accept().expect("accept framing request");
        read_request(&mut framing);
        framing
            .write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Length: 1\r\n\r\n0\r\n\r\n")
            .expect("write ambiguous response");

        let (mut interim, _) = listener.accept().expect("accept interim request");
        read_request(&mut interim);
        for _ in 0..9 {
            interim
                .write_all(b"HTTP/1.1 100 Continue\r\n\r\n")
                .expect("write interim response");
        }
        interim
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .expect("write final response");

        let (mut header, _) = listener.accept().expect("accept invalid header request");
        read_request(&mut header);
        header
            .write_all(b"HTTP/1.1 200 OK\r\nBad Name: value\r\nContent-Length: 0\r\n\r\n")
            .expect("write invalid response header");

        let (mut status, _) = listener.accept().expect("accept invalid status request");
        read_request(&mut status);
        status
            .write_all(b"HTTP/1.1 20 OK\r\nContent-Length: 0\r\n\r\n")
            .expect("write invalid response status");
    });

    let (exit, stdout, stderr) = run_program(
        "http-hostile-responses",
        &format!(
            r#"program HttpHostileResponses;

uses Std.Console as Console, Std.Net.Http as Http;

procedure ExpectError(Url: string; Interim: boolean);
begin
  case Http.Send(Http.Request.Get(Url)) of
    when Ok(_): panic('hostile HTTP response was accepted');
    when Error(const Message):
    begin
      if Interim then
      begin
        if not (Message is Http.HttpError.LimitExceeded('Std.Net.Http.Head.ReadFinal', 8)) then panic(Message.Message()); end if;
      end;
      else begin
        if not (Message is Http.HttpError.InvalidMessage(_)) then panic(Message.Message()); end if;
      end; end if;
    end;
  end case;
end procedure;

begin
  ExpectError('http://127.0.0.1:{port}/framing', false);
  ExpectError('http://127.0.0.1:{port}/interim', true);
  ExpectError('http://127.0.0.1:{port}/header', false);
  ExpectError('http://127.0.0.1:{port}/status', false);
  Console.WriteLn('ok');
end.
"#
        ),
    );
    assert_eq!(exit, 0, "stderr: {stderr}");
    server.join().expect("hostile response fixture must finish");
    assert_eq!(stdout, "ok\n");
}

#[test]
fn http_client_rejects_chunk_size_integer_overflow() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind chunk overflow fixture");
    let port = listener
        .local_addr()
        .expect("chunk overflow address")
        .port();
    let server = listener.spawn(move |listener| {
        for size in ["8000000000000000", "10000000000000000"] {
            let (mut stream, _) = listener.accept().expect("accept chunk overflow request");
            read_request(&mut stream);
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{size}\r\n"
            )
            .expect("write overflowing chunk size");
        }
    });

    let (exit, stdout, stderr) = run_program(
        "http-chunk-size-overflow",
        &format!(
            r#"program HttpChunkSizeOverflow;

uses Std.Console as Console, Std.Net.Http as Http;

procedure ExpectOverflow(Path: string);
begin
  case Http.Send(Http.Request.Get('http://127.0.0.1:{port}/' + Path)) of
    when Ok(_): panic('overflowing HTTP chunk size was accepted');
    when Error(const Message):
    begin
      if not (Message is Http.HttpError.InvalidMessage(_)) then panic(Message.Message()); end if;
    end;
  end case;
end procedure;

begin
  ExpectOverflow('negative');
  ExpectOverflow('zero');
  Console.WriteLn('ok');
end.
"#
        ),
    );
    assert_eq!(exit, 0, "stderr: {stderr}");
    server.join().expect("chunk overflow fixture must finish");
    assert_eq!(stdout, "ok\n");
}
