use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::*;

mod decimal_framing;
mod https;
mod listener_address;
mod parallel_echo;
mod server_loop;

fn unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve server fixture port");
    listener.local_addr().expect("reserved address").port()
}

fn start_server(
    name: &str,
    source_text: String,
) -> (PathBuf, std::thread::JoinHandle<(i32, String, String)>) {
    let cwd = create_temp_dir(name);
    let server = spawn_server(&cwd, source_text);
    (cwd, server)
}

fn spawn_server(cwd: &Path, source_text: String) -> std::thread::JoinHandle<(i32, String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let source = cwd.join("main.fpas");
    write_text(&source, &source_text);
    let thread_cwd = cwd.to_path_buf();
    std::thread::spawn(move || {
        support::run_cli_args_and_capture_output(
            &[
                String::from("run"),
                String::from("--std-lib"),
                root.join("lib").to_string_lossy().into_owned(),
                source.to_string_lossy().into_owned(),
            ],
            &thread_cwd,
        )
    })
}

fn connect_when_ready(port: u16) -> TcpStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => {
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .expect("set fixture read timeout");
                return stream;
            }
            Err(error) if Instant::now() < deadline => {
                drop(error);
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("connect to FPAS HTTP server: {error}"),
        }
    }
}

fn exchange(port: u16, fragments: &[&[u8]]) -> Vec<u8> {
    let mut stream = connect_when_ready(port);
    for fragment in fragments {
        stream.write_all(fragment).expect("write HTTP request");
    }
    stream
        .shutdown(Shutdown::Write)
        .expect("finish HTTP request");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("read HTTP response");
    response
}

fn finish_server(
    cwd: PathBuf,
    server: std::thread::JoinHandle<(i32, String, String)>,
) -> (String, String) {
    let (exit, stdout, stderr) = server.join().expect("FPAS HTTP server must finish");
    std::fs::remove_dir_all(cwd).expect("temporary directory must be removed");
    assert_eq!(exit, 0, "stderr: {stderr}");
    (stdout, stderr)
}

#[test]
fn fpas_http_server_accepts_get_and_writes_response() {
    let port = unused_port();
    let (cwd, server) = start_server(
        "http-server-get",
        format!(
            "program HttpServerGet;\n\nuses Std.Console as Console; uses Std.Http as Http; uses Std.Net as Net; uses Std.Net.Utf8 as Utf8;\n\nbegin\n  case Net.Listen('127.0.0.1', {port}) of\n    when Result.Ok(const ListenerValue):\n    begin\n      case Net.Accept(ListenerValue) of\n        when Result.Ok(const Connection):\n        begin\n          case Net.SetTimeout(Connection, 2000) of\n            when Result.Ok(_):\n            begin null;\n            end;\n            when Result.Error(const Message): panic(Message);\n          end case;\n          case Http.ReadRequest(Connection, 4096, 1024) of\n            when Result.Ok(const RequestValue):\n            begin\n              if (RequestValue.Method <> 'GET') or (RequestValue.Target <> '/hello?name=fpas') then\n              begin\n                panic('unexpected request');\n              end; end if;\n              mutable var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(200, 'OK');\n              ResponseValue.Headers := [Http.Header.Create('Content-Type', 'text/plain')];\n              ResponseValue.Body := Utf8.Encode('hello from fpas');\n              case Http.WriteResponse(Connection, ResponseValue) of\n                when Result.Ok(_):\n                begin null;\n                end;\n                when Result.Error(const Message): panic(Message);\n              end case;\n            end;\n            when Result.Error(const Message): panic(Message);\n          end case;\n          case Net.Close(Connection) of\n            when Result.Ok(_):\n            begin null;\n            end;\n            when Result.Error(const Message): panic(Message);\n          end case;\n        end;\n        when Result.Error(const Message): panic(Message);\n      end case;\n      case Net.CloseListener(ListenerValue) of\n        when Result.Ok(_): Console.WriteLn('served');\n        when Result.Error(const Message): panic(Message);\n      end case;\n    end;\n    when Result.Error(const Message): panic(Message);\n  end case;\nend program;\n"
        ),
    );

    let response = exchange(
        port,
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
    let port = unused_port();
    let (cwd, server) = start_server(
        "http-server-post",
        format!(
            "program HttpServerPost;\n\nuses Std.Http as Http; uses Std.Net as Net; uses Std.Net.Utf8 as Utf8;\n\nbegin\n  case Net.Listen('127.0.0.1', {port}) of\n    when Result.Ok(const ListenerValue):\n    begin\n      case Net.Accept(ListenerValue) of\n        when Result.Ok(const Connection):\n        begin\n          case Net.SetTimeout(Connection, 2000) of\n            when Result.Ok(_):\n            begin null;\n            end;\n            when Result.Error(const Message): panic(Message);\n          end case;\n          case Http.ReadRequest(Connection, 4096, 1024) of\n            when Result.Ok(const RequestValue):\n            begin\n              case Utf8.Decode(RequestValue.Body) of\n                when Result.Ok(const Text):\n                begin\n                  mutable var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(201, 'Created');\n                  ResponseValue.Body := Utf8.Encode(RequestValue.Target + ':' + Text);\n                  case Http.WriteResponse(Connection, ResponseValue) of\n                    when Result.Ok(_):\n                    begin null;\n                    end;\n                    when Result.Error(const Message): panic(Message);\n                  end case;\n                end;\n                when Result.Error(const Message): panic(Message);\n              end case;\n            end;\n            when Result.Error(const Message): panic(Message);\n          end case;\n          case Net.Close(Connection) of\n            when Result.Ok(_):\n            begin null;\n            end;\n            when Result.Error(const Message): panic(Message);\n          end case;\n        end;\n        when Result.Error(const Message): panic(Message);\n      end case;\n      case Net.CloseListener(ListenerValue) of\n        when Result.Ok(_):\n        begin null;\n        end;\n        when Result.Error(const Message): panic(Message);\n      end case;\n    end;\n    when Result.Error(const Message): panic(Message);\n  end case;\nend program;\n"
        ),
    );

    let response = exchange(
        port,
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
    let port = unused_port();
    let (cwd, server) = start_server(
        "http-server-hostile",
        format!(
            "program HttpServerHostile;\n\nuses Std.Http as Http; uses Std.Net as Net; uses Std.Net.Utf8 as Utf8; uses Std.Str as Str;\n\nbegin\n  case Net.Listen('127.0.0.1', {port}) of\n    when Result.Ok(const ListenerValue):\n    begin\n      case Net.Accept(ListenerValue) of\n        when Result.Ok(const Connection):\n        begin\n          case Http.ReadRequest(Connection, 4096, 1024) of\n            when Result.Ok(_): panic('ambiguous framing was accepted');\n            when Result.Error(const Message):\n            begin\n              if not Str.Contains(Message, 'both Transfer-Encoding and Content-Length') then\n              begin\n                panic(Message);\n              end; end if;\n              mutable var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(400, 'Bad Request');\n              ResponseValue.Body := Utf8.Encode('rejected');\n              case Http.WriteResponse(Connection, ResponseValue) of\n                when Result.Ok(_):\n                begin null;\n                end;\n                when Result.Error(const WriteMessage): panic(WriteMessage);\n              end case;\n            end;\n          end case;\n          case Net.Close(Connection) of\n            when Result.Ok(_):\n            begin null;\n            end;\n            when Result.Error(const Message): panic(Message);\n          end case;\n        end;\n        when Result.Error(const Message): panic(Message);\n      end case;\n      case Net.CloseListener(ListenerValue) of\n        when Result.Ok(_):\n        begin null;\n        end;\n        when Result.Error(const Message): panic(Message);\n      end case;\n    end;\n    when Result.Error(const Message): panic(Message);\n  end case;\nend program;\n"
        ),
    );

    let response = exchange(
        port,
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
    let port = unused_port();
    let (cwd, server) = start_server(
        "http-server-body-limit",
        format!(
            "program HttpServerBodyLimit;\n\nuses Std.Http as Http; uses Std.Net as Net; uses Std.Net.Utf8 as Utf8; uses Std.Str as Str;\n\nbegin\n  case Net.Listen('127.0.0.1', {port}) of\n    when Result.Ok(const ListenerValue):\n    begin\n      case Net.Accept(ListenerValue) of\n        when Result.Ok(const Connection):\n        begin\n          case Http.ReadRequest(Connection, 4096, 4) of\n            when Result.Ok(_): panic('oversized request body was accepted');\n            when Result.Error(const Message):\n            begin\n              if not Str.Contains(Message, 'MaxBodyBytes') then\n              begin\n                panic(Message);\n              end; end if;\n              mutable var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(413, 'Content Too Large');\n              ResponseValue.Body := Utf8.Encode('too large');\n              case Http.WriteResponse(Connection, ResponseValue) of\n                when Result.Ok(_):\n                begin null;\n                end;\n                when Result.Error(const WriteMessage): panic(WriteMessage);\n              end case;\n            end;\n          end case;\n          case Net.Close(Connection) of\n            when Result.Ok(_):\n            begin null;\n            end;\n            when Result.Error(const Message): panic(Message);\n          end case;\n        end;\n        when Result.Error(const Message): panic(Message);\n      end case;\n      case Net.CloseListener(ListenerValue) of\n        when Result.Ok(_):\n        begin null;\n        end;\n        when Result.Error(const Message): panic(Message);\n      end case;\n    end;\n    when Result.Error(const Message): panic(Message);\n  end case;\nend program;\n"
        ),
    );

    let response = exchange(
        port,
        &[b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\nhello"],
    );
    finish_server(cwd, server);

    assert_eq!(
        response,
        b"HTTP/1.1 413 Content Too Large\r\nConnection: close\r\nContent-Length: 9\r\n\r\ntoo large"
    );
}
