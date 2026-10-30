use super::{exchange, finish_server, start_server};

#[test]
fn fpas_http_server_loop_dispatches_multiple_requests() {
    let (cwd, server) = start_server(
        "http-server-loop",
        String::from(
            r#"program HttpServerLoop;

uses Std.Console as Console, Std.Net.Http as Http, Std.Net.Sockets as Net, Std.Text.Utf8 as Utf8;

function Handle(RequestValue: Http.ServerRequest): Http.ServerResponse;
begin
  var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(200, 'OK');
  ResponseValue.Body := Utf8.Encode(RequestValue.Target);
  return ResponseValue;
end function;

begin
  case Net.Listen('127.0.0.1', 0) of
    when Ok(const ListenerValue):
    begin
      Console.WriteLn('FPAS_SERVER_FIXTURE_PORT=', Net.ListenerLocalAddress(ListenerValue).Unwrap().Port);
      var Options: Http.ServerOptions := Http.ServerOptions.Create();
      Options.MaxConcurrentRequests := 2;
      Options.MaxRequests := 2;
      case Http.Serve(ListenerValue, Options, Handle) of
        when Ok(_):
        begin
        end;
        when Error(const Message): panic(Message.Message());
      end case;
      case Net.CloseListener(ListenerValue) of
        when Ok(_): Console.WriteLn('served');
        when Error(const Message): panic(Message.Message());
      end case;
    end;
    when Error(const Message): panic(Message.Message());
  end case;
end.
"#,
        ),
    );

    let port = server.port();
    let first = std::thread::spawn(move || {
        exchange(port, &[b"GET /first HTTP/1.1\r\nHost: localhost\r\n\r\n"])
    });
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
    let (cwd, server) = start_server(
        "http-server-loop-invalid-request",
        String::from(
            r#"program HttpServerLoopInvalidRequest;

uses Std.Console as Console, Std.Net.Http as Http, Std.Net.Sockets as Net;

function Handle(_RequestValue: Http.ServerRequest): Http.ServerResponse;
begin
  return Http.ServerResponse.Create(204, 'No Content');
end function;

begin
  case Net.Listen('127.0.0.1', 0) of
    when Ok(const ListenerValue):
    begin
      Console.WriteLn('FPAS_SERVER_FIXTURE_PORT=', Net.ListenerLocalAddress(ListenerValue).Unwrap().Port);
      var Options: Http.ServerOptions := Http.ServerOptions.Create();
      Options.MaxConcurrentRequests := 2;
      Options.MaxRequests := 2;
      case Http.Serve(ListenerValue, Options, Handle) of
        when Ok(_):
        begin
        end;
        when Error(const Message): panic(Message.Message());
      end case;
      case Net.CloseListener(ListenerValue) of
        when Ok(_):
        begin
        end;
        when Error(const Message): panic(Message.Message());
      end case;
    end;
    when Error(const Message): panic(Message.Message());
  end case;
end.
"#,
        ),
    );

    let port = server.port();
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
    let (cwd, server) = start_server(
        "http-server-loop-invalid-options",
        String::from(
            r#"program HttpServerLoopInvalidOptions;

uses Std.Console as Console, Std.Net.Http as Http, Std.Net.Sockets as Net;

function Handle(_RequestValue: Http.ServerRequest): Http.ServerResponse;
begin
  return Http.ServerResponse.Create(204, 'No Content');
end function;

begin
  case Net.Listen('127.0.0.1', 0) of
    when Ok(const ListenerValue):
    begin
      Console.WriteLn('FPAS_SERVER_FIXTURE_PORT=', Net.ListenerLocalAddress(ListenerValue).Unwrap().Port);
      var Options: Http.ServerOptions := Http.ServerOptions.Create();
      Options.MaxConcurrentRequests := 0;
      case Http.Serve(ListenerValue, Options, Handle) of
        when Ok(_): panic('invalid server options were accepted');
        when Error(const Message):
        begin
          if not (Message is Http.HttpError.InvalidArgument('MaxConcurrentRequests', _)) then
          begin
            panic(Message.Message());
          end; end if;
        end;
      end case;
      case Net.CloseListener(ListenerValue) of
        when Ok(_): Console.WriteLn('rejected');
        when Error(const Message): panic(Message.Message());
      end case;
    end;
    when Error(const Message): panic(Message.Message());
  end case;
end.
"#,
        ),
    );

    let (stdout, _) = finish_server(cwd, server);

    assert_eq!(stdout, "rejected\n");
}
