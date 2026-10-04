use std::time::Duration;

use super::{exchange, finish_server, start_server, unused_port};

#[test]
fn fpas_http_server_loop_dispatches_multiple_requests() {
    let port = unused_port();
    let (cwd, server) = start_server(
        "http-server-loop",
        format!(
            r#"program HttpServerLoop;

uses Std.Console as Console; uses Std.Http as Http; uses Std.Net as Net; uses Std.Net.Utf8 as Utf8;

function Handle(RequestValue: Http.ServerRequest): Http.ServerResponse;
begin
   var ResponseValue: Http.ServerResponse := Http.ServerResponseCreate(200, 'OK');
  ResponseValue.Body := Utf8.Encode(RequestValue.Target);
  return ResponseValue;
end function;

begin
  case Net.Listen('127.0.0.1', {port}) of
    when Result.Ok(const ListenerValue):
    begin
       var Options: Http.ServerOptions := Http.ServerOptionsCreate();
      Options.MaxConcurrentRequests := 2;
      Options.MaxRequests := 2;
      case Http.Serve(ListenerValue, Options, Handle) of
        when Result.Ok(_):
        begin null;
        end;
        when Result.Error(const Message): panic(Message);
      end case;
      case Net.CloseListener(ListenerValue) of
        when Result.Ok(_): Console.WriteLn('served');
        when Result.Error(const Message): panic(Message);
      end case;
    end;
    when Result.Error(const Message): panic(Message);
  end case;
end program;
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

uses Std.Http as Http; uses Std.Net as Net;

function Handle(_RequestValue: Http.ServerRequest): Http.ServerResponse;
begin
  return Http.ServerResponseCreate(204, 'No Content');
end function;

begin
  case Net.Listen('127.0.0.1', {port}) of
    when Result.Ok(const ListenerValue):
    begin
       var Options: Http.ServerOptions := Http.ServerOptionsCreate();
      Options.MaxConcurrentRequests := 2;
      Options.MaxRequests := 2;
      case Http.Serve(ListenerValue, Options, Handle) of
        when Result.Ok(_):
        begin null;
        end;
        when Result.Error(const Message): panic(Message);
      end case;
      case Net.CloseListener(ListenerValue) of
        when Result.Ok(_):
        begin null;
        end;
        when Result.Error(const Message): panic(Message);
      end case;
    end;
    when Result.Error(const Message): panic(Message);
  end case;
end program;
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

uses Std.Console as Console; uses Std.Http as Http; uses Std.Net as Net; uses Std.Str as Str;

function Handle(_RequestValue: Http.ServerRequest): Http.ServerResponse;
begin
  return Http.ServerResponseCreate(204, 'No Content');
end function;

begin
  case Net.Listen('127.0.0.1', {port}) of
    when Result.Ok(const ListenerValue):
    begin
       var Options: Http.ServerOptions := Http.ServerOptionsCreate();
      Options.MaxConcurrentRequests := 0;
      case Http.Serve(ListenerValue, Options, Handle) of
        when Result.Ok(_): panic('invalid server options were accepted');
        when Result.Error(const Message):
        begin
          if not Str.Contains(Message, 'MaxConcurrentRequests') then
          begin
            panic(Message);
          end; end if;
        end;
      end case;
      case Net.CloseListener(ListenerValue) of
        when Result.Ok(_): Console.WriteLn('rejected');
        when Result.Error(const Message): panic(Message);
      end case;
    end;
    when Result.Error(const Message): panic(Message);
  end case;
end program;
"#
        ),
    );

    let (stdout, _) = finish_server(cwd, server);

    assert_eq!(stdout, "rejected\n");
}
