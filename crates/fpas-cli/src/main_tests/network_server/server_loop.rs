use std::time::Duration;

use super::{exchange, finish_server, start_server, unused_port};

#[test]
fn fpas_http_server_loop_dispatches_multiple_requests() {
    let port = unused_port();
    let (cwd, server) = start_server(
        "http-server-loop",
        format!(
            r#"program HttpServerLoop;

uses Std.Console, Std.Http, Std.Net, Std.Net.Utf8;

function Handle(RequestValue: ServerRequest): ServerResponse;
begin
  mutable var ResponseValue: ServerResponse := ServerResponse.Create(200, 'OK');
  ResponseValue.Body := Std.Net.Utf8.Encode(RequestValue.Target);
  return ResponseValue
end;

begin
  case Listen('127.0.0.1', {port}) of
    Ok(ListenerValue):
    begin
      mutable var Options: ServerOptions := ServerOptions.Create();
      Options.MaxConcurrentRequests := 2;
      Options.MaxRequests := 2;
      case Serve(ListenerValue, Options, Handle) of
        Ok(_):
        begin
        end;
        Error(Message): panic(Message)
      end;
      case CloseListener(ListenerValue) of
        Ok(_): WriteLn('served');
        Error(Message): panic(Message)
      end
    end;
    Error(Message): panic(Message)
  end
end.
"#
        ),
    );

    let first = std::thread::spawn(move || {
        exchange(port, &[b"GET /first HTTP/1.1\r\nHost: localhost\r\n\r\n"])
    });
    std::thread::sleep(Duration::from_millis(20));
    let second = std::thread::spawn(move || {
        exchange(port, &[b"GET /second HTTP/1.1\r\nHost: localhost\r\n\r\n"])
    });

    let first_response = first.join().expect("first HTTP client must finish");
    let second_response = second.join().expect("second HTTP client must finish");
    let (stdout, _) = finish_server(cwd, server);

    assert_eq!(stdout, "served\n");
    assert_eq!(
        first_response,
        b"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 6\r\n\r\n/first"
    );
    assert_eq!(
        second_response,
        b"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 7\r\n\r\n/second"
    );
}

#[test]
fn fpas_http_server_loop_isolates_malformed_requests() {
    let port = unused_port();
    let (cwd, server) = start_server(
        "http-server-loop-invalid-request",
        format!(
            r#"program HttpServerLoopInvalidRequest;

uses Std.Http, Std.Net;

function Handle(_RequestValue: ServerRequest): ServerResponse;
begin
  return ServerResponse.Create(204, 'No Content')
end;

begin
  case Listen('127.0.0.1', {port}) of
    Ok(ListenerValue):
    begin
      mutable var Options: ServerOptions := ServerOptions.Create();
      Options.MaxConcurrentRequests := 2;
      Options.MaxRequests := 2;
      case Serve(ListenerValue, Options, Handle) of
        Ok(_):
        begin
        end;
        Error(Message): panic(Message)
      end;
      case CloseListener(ListenerValue) of
        Ok(_):
        begin
        end;
        Error(Message): panic(Message)
      end
    end;
    Error(Message): panic(Message)
  end
end.
"#
        ),
    );

    let invalid = std::thread::spawn(move || {
        exchange(port, &[b"GET / HTTP/1.1\r\nHost: one\r\nHost: two\r\n\r\n"])
    });
    let valid =
        std::thread::spawn(move || exchange(port, &[b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n"]));

    let invalid_response = invalid.join().expect("invalid HTTP client must finish");
    let valid_response = valid.join().expect("valid HTTP client must finish");
    finish_server(cwd, server);

    assert_eq!(
        invalid_response,
        b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    );
    assert_eq!(
        valid_response,
        b"HTTP/1.1 204 No Content\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    );
}

#[test]
fn fpas_http_server_loop_rejects_invalid_options_before_accepting() {
    let port = unused_port();
    let (cwd, server) = start_server(
        "http-server-loop-invalid-options",
        format!(
            r#"program HttpServerLoopInvalidOptions;

uses Std.Console, Std.Http, Std.Net, Std.Str;

function Handle(_RequestValue: ServerRequest): ServerResponse;
begin
  return ServerResponse.Create(204, 'No Content')
end;

begin
  case Listen('127.0.0.1', {port}) of
    Ok(ListenerValue):
    begin
      mutable var Options: ServerOptions := ServerOptions.Create();
      Options.MaxConcurrentRequests := 0;
      case Serve(ListenerValue, Options, Handle) of
        Ok(_): panic('invalid server options were accepted');
        Error(Message):
        begin
          if not Std.Str.Contains(Message, 'MaxConcurrentRequests') then
          begin
            panic(Message)
          end
        end
      end;
      case CloseListener(ListenerValue) of
        Ok(_): WriteLn('rejected');
        Error(Message): panic(Message)
      end
    end;
    Error(Message): panic(Message)
  end
end.
"#
        ),
    );

    let (stdout, _) = finish_server(cwd, server);

    assert_eq!(stdout, "rejected\n");
}
