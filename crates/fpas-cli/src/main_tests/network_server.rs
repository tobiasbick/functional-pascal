use super::process::TempDir;
use super::*;

mod decimal_framing;
mod fixture;
mod https;
mod listener_address;
mod parallel_echo;
mod server_loop;
use fixture::{connect_when_ready, exchange, finish_server, spawn_server, start_server};

#[test]
fn fpas_http_server_accepts_get_and_writes_response() {
    let (cwd, server) = start_server(
        "http-server-get",
        String::from(
            r#"program HttpServerGet;

uses Std.Console as Console, Std.Net.Http as Http, Std.Net.Sockets as Net, Std.Text.Utf8 as Utf8;

begin
  case Net.Listen('127.0.0.1', 0) of
    when Ok(const ListenerValue):
    begin
      Console.WriteLn('FPAS_SERVER_FIXTURE_PORT=', Net.ListenerLocalAddress(ListenerValue).Unwrap().Port);
      case Net.Accept(ListenerValue) of
        when Ok(const Connection):
        begin
          case Net.SetTimeout(Connection, 2000) of
            when Ok(_):
            begin
            end;
            when Error(const Message): panic(Message.Message());
          end case;
          case Http.ReadRequest(Connection, 4096, 1024) of
            when Ok(const RequestValue):
            begin
              if (RequestValue.Method <> 'GET') or (RequestValue.Target <> '/hello?name=fpas') then
              begin
                panic('unexpected request');
              end; end if;
              var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(200, 'OK');
              ResponseValue.Headers := [Http.Header.Create('Content-Type', 'text/plain')];
              ResponseValue.Body := Utf8.Encode('hello from fpas');
              case Http.WriteResponse(Connection, ResponseValue) of
                when Ok(_):
                begin
                end;
                when Error(const Message): panic(Message.Message());
              end case;
            end;
            when Error(const Message): panic(Message.Message());
          end case;
          case Net.Close(Connection) of
            when Ok(_):
            begin
            end;
            when Error(const Message): panic(Message.Message());
          end case;
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

    let response = exchange(
        server.port(),
        &[b"GET /hello?name=fpas HTTP/1.1\r\nHost: localhost\r\n\r\n"],
    );
    let (stdout, _) = finish_server(cwd, server);

    assert_eq!(stdout, "served\n");
    assert_eq!(
        response,
        b"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 15\r\nContent-Type: text/plain\r\n\r\nhello from fpas"
    );
}

#[test]
fn fpas_http_server_reads_fragmented_post_body() {
    let (cwd, server) = start_server(
        "http-server-post",
        String::from(
            r#"program HttpServerPost;

uses Std.Console as Console, Std.Net.Http as Http, Std.Net.Sockets as Net, Std.Text.Utf8 as Utf8;

begin
  case Net.Listen('127.0.0.1', 0) of
    when Ok(const ListenerValue):
    begin
      Console.WriteLn('FPAS_SERVER_FIXTURE_PORT=', Net.ListenerLocalAddress(ListenerValue).Unwrap().Port);
      case Net.Accept(ListenerValue) of
        when Ok(const Connection):
        begin
          case Net.SetTimeout(Connection, 2000) of
            when Ok(_):
            begin
            end;
            when Error(const Message): panic(Message.Message());
          end case;
          case Http.ReadRequest(Connection, 4096, 1024) of
            when Ok(const RequestValue):
            begin
              case Utf8.Decode(RequestValue.Body) of
                when Ok(const Text):
                begin
                  var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(201, 'Created');
                  ResponseValue.Body := Utf8.Encode(RequestValue.Target + ':' + Text);
                  case Http.WriteResponse(Connection, ResponseValue) of
                    when Ok(_):
                    begin
                    end;
                    when Error(const Message): panic(Message.Message());
                  end case;
                end;
                when Error(const Message): panic(Message.Message());
              end case;
            end;
            when Error(const Message): panic(Message.Message());
          end case;
          case Net.Close(Connection) of
            when Ok(_):
            begin
            end;
            when Error(const Message): panic(Message.Message());
          end case;
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

    let response = exchange(
        server.port(),
        &[
            b"POST /items HTTP/1.1\r\nHost: localhost\r\nContent-Length: 7\r\n\r\n",
            b"pay",
            b"load",
        ],
    );
    finish_server(cwd, server);

    assert_eq!(
        response,
        b"HTTP/1.1 201 Created\r\nConnection: close\r\nContent-Length: 14\r\n\r\n/items:payload"
    );
}

#[test]
fn fpas_http_server_rejects_ambiguous_request_framing() {
    let (cwd, server) = start_server(
        "http-server-hostile",
        String::from(
            r#"program HttpServerHostile;

uses Std.Console as Console, Std.Net.Http as Http, Std.Net.Sockets as Net, Std.Text.Utf8 as Utf8;

begin
  case Net.Listen('127.0.0.1', 0) of
    when Ok(const ListenerValue):
    begin
      Console.WriteLn('FPAS_SERVER_FIXTURE_PORT=', Net.ListenerLocalAddress(ListenerValue).Unwrap().Port);
      case Net.Accept(ListenerValue) of
        when Ok(const Connection):
        begin
          case Http.ReadRequest(Connection, 4096, 1024) of
            when Ok(_): panic('ambiguous framing was accepted');
            when Error(const Message):
            begin
              if not (Message is Http.HttpError.InvalidMessage(_)) then
              begin
                panic(Message.Message());
              end; end if;
              var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(400, 'Bad Request');
              ResponseValue.Body := Utf8.Encode('rejected');
              case Http.WriteResponse(Connection, ResponseValue) of
                when Ok(_):
                begin
                end;
                when Error(const WriteMessage): panic(WriteMessage.Message());
              end case;
            end;
          end case;
          case Net.Close(Connection) of
            when Ok(_):
            begin
            end;
            when Error(const Message): panic(Message.Message());
          end case;
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

    let response = exchange(
        server.port(),
        &[b"POST / HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\nContent-Length: 1\r\n\r\n0\r\n\r\n"],
    );
    finish_server(cwd, server);

    assert_eq!(
        response,
        b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Length: 8\r\n\r\nrejected"
    );
}

#[test]
fn fpas_http_server_enforces_request_body_limit() {
    let (cwd, server) = start_server(
        "http-server-body-limit",
        String::from(
            r#"program HttpServerBodyLimit;

uses Std.Console as Console, Std.Net.Http as Http, Std.Net.Sockets as Net, Std.Text.Utf8 as Utf8;

begin
  case Net.Listen('127.0.0.1', 0) of
    when Ok(const ListenerValue):
    begin
      Console.WriteLn('FPAS_SERVER_FIXTURE_PORT=', Net.ListenerLocalAddress(ListenerValue).Unwrap().Port);
      case Net.Accept(ListenerValue) of
        when Ok(const Connection):
        begin
          case Http.ReadRequest(Connection, 4096, 4) of
            when Ok(_): panic('oversized request body was accepted');
            when Error(const Message):
            begin
              if not (Message is Http.HttpError.LimitExceeded('Std.Net.Http.Server.ReadRequest', _)) then
              begin
                panic(Message.Message());
              end; end if;
              var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(413, 'Content Too Large');
              ResponseValue.Body := Utf8.Encode('too large');
              case Http.WriteResponse(Connection, ResponseValue) of
                when Ok(_):
                begin
                end;
                when Error(const WriteMessage): panic(WriteMessage.Message());
              end case;
            end;
          end case;
          case Net.Close(Connection) of
            when Ok(_):
            begin
            end;
            when Error(const Message): panic(Message.Message());
          end case;
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

    let response = exchange(
        server.port(),
        &[b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\nhello"],
    );
    finish_server(cwd, server);

    assert_eq!(
        response,
        b"HTTP/1.1 413 Content Too Large\r\nConnection: close\r\nContent-Length: 9\r\n\r\ntoo large"
    );
}
