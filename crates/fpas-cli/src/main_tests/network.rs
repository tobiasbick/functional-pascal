use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use super::*;

fn read_http_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut request = Vec::new();
    loop {
        let mut chunk = [0_u8; 2048];
        let count = stream.read(&mut chunk).expect("read HTTP request");
        if count == 0 {
            return request;
        }
        request.extend_from_slice(&chunk[..count]);
        let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                line.strip_prefix("Content-Length: ")
                    .and_then(|value| value.parse::<usize>().ok())
            })
            .expect("content length");
        if request.len() >= header_end + 4 + content_length {
            return request;
        }
    }
}

#[test]
fn http_client_sends_request_and_decodes_chunked_response() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local HTTP fixture");
    let port = listener.local_addr().expect("fixture address").port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept HTTP client");
        let request = read_http_request(&mut stream);
        let request = String::from_utf8(request).expect("HTTP request is UTF-8");
        assert!(request.starts_with("POST /v1/chat HTTP/1.1\r\n"));
        assert!(request.contains("Content-Length: 4\r\n"));
        assert!(request.contains("X-Test: yes\r\n"));
        assert!(request.ends_with("\r\nping"));

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n",
            )
            .expect("write chunked HTTP response");
    });

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let cwd = create_temp_dir("http-client-roundtrip");
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        &format!(
            r#"program HttpClientRoundtrip;

uses Std.Console as Console;
uses Std.Http as Http;
uses Std.Net.Utf8 as Utf8;

begin
  mutable var RequestValue: Http.Request := Http.Request.Create('POST', 'http://127.0.0.1:{port}/v1/chat');
  RequestValue.Headers := [Http.Header.Create('X-Test', 'yes')];
  RequestValue.Body := Utf8.Encode('ping');
  case Http.Send(RequestValue) of
    when Result.Ok(const ResponseValue):
      begin
        Console.WriteLn(ResponseValue.StatusCode);
        case Http.BodyText(ResponseValue) of
          when Result.Ok(const Text):
            Console.WriteLn(Text);
          when Result.Error(const Message):
            panic(Message);
        end case;

        case Http.HeaderValue(ResponseValue, 'content-type') of
          when Option.Some(const Value):
            Console.WriteLn(Value);
          when Option.None:
            panic('missing content type');
        end case;
      end;
    when Result.Error(const Message):
      panic(Message);
  end case;
end program;
"#
        ),
    );

    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("run"),
            String::from("--std-lib"),
            root.join("lib").to_string_lossy().into_owned(),
            source.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    std::fs::remove_dir_all(&cwd).expect("temporary directory must be removed");
    assert_eq!(exit, 0, "stderr: {stderr}");
    server.join().expect("HTTP fixture must finish");
    assert_eq!(stdout, "200\nhello world\ntext/plain\n");
}

#[test]
fn http_client_supports_standard_extension_and_head_methods() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local HTTP method fixture");
    let port = listener.local_addr().expect("fixture address").port();
    let server = std::thread::spawn(move || {
        let expected = [
            ("GET", "/get"),
            ("POST", "/post"),
            ("PUT", "/put"),
            ("PATCH", "/patch"),
            ("DELETE", "/delete"),
            ("HEAD", "/head"),
            ("OPTIONS", "/options"),
            ("PROPFIND", "/webdav"),
        ];
        for (method, path) in expected {
            let (mut stream, _) = listener.accept().expect("accept HTTP method client");
            let request = read_http_request(&mut stream);
            let request = String::from_utf8(request).expect("HTTP method request is UTF-8");
            assert!(
                request.starts_with(&format!("{method} {path} HTTP/1.1\r\n")),
                "request: {request:?}"
            );
            if method == "HEAD" {
                stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\n")
                    .expect("write HEAD response");
            } else {
                stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
                    )
                    .expect("write method response");
            }
        }
    });

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let cwd = create_temp_dir("http-client-methods");
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        &format!(
            r#"program HttpClientMethods;

uses Std.Arrays as Arrays;
uses Std.Console as Console;
uses Std.Http as Http;
uses Std.Str as Str;

procedure Expect(RequestValue: Http.Request; ExpectedBodyLength: integer);
begin
  case Http.Send(RequestValue) of
    when Result.Ok(const ResponseValue):
      begin
        if ResponseValue.StatusCode <> 200 then
          begin
            panic('unexpected HTTP status');
          end;
        end if;

        if Arrays.Length(ResponseValue.Body) <> ExpectedBodyLength then
          begin
            panic('unexpected HTTP body length');
          end;
        end if;
      end;
    when Result.Error(const Message):
      begin
        panic(Message);
      end;
  end case;
end procedure;

begin
  var BaseUrl: string := 'http://127.0.0.1:{port}';
  Expect(Http.Request.Get(BaseUrl + '/get'), 2);
  Expect(Http.Request.Post(BaseUrl + '/post'), 2);
  Expect(Http.Request.Put(BaseUrl + '/put'), 2);
  Expect(Http.Request.Patch(BaseUrl + '/patch'), 2);
  Expect(Http.Request.Delete(BaseUrl + '/delete'), 2);
  Expect(Http.Request.Head(BaseUrl + '/head'), 0);
  Expect(Http.Request.Options(BaseUrl + '/options'), 2);
  Expect(Http.Request.Create('PROPFIND', BaseUrl + '/webdav'), 2);
  case Http.Send(Http.Request.Create('BAD@METHOD', BaseUrl + '/invalid')) of
    when Result.Ok(const ResponseValue):
      begin
        panic('invalid HTTP method was accepted');
      end;
    when Result.Error(const Message):
      begin
        if not Str.Contains(Message, 'RFC 9110 token') then
          begin
            panic(Message);
          end;
        end if;
      end;
  end case;

  Console.WriteLn('ok');
end program;
"#
        ),
    );

    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("run"),
            String::from("--std-lib"),
            root.join("lib").to_string_lossy().into_owned(),
            source.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    std::fs::remove_dir_all(&cwd).expect("temporary directory must be removed");
    assert_eq!(exit, 0, "stderr: {stderr}");
    server.join().expect("HTTP method fixture must finish");
    assert_eq!(stdout, "ok\n");
}

#[test]
fn openai_compatible_client_sends_configured_chat_completion() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local OpenAI fixture");
    let port = listener.local_addr().expect("fixture address").port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept OpenAI client");
        let request = read_http_request(&mut stream);

        let header_end = request
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("request header terminator");
        let headers = String::from_utf8(request[..header_end].to_vec()).expect("request headers");
        assert!(headers.starts_with("POST /v1/chat/completions HTTP/1.1\r\n"));
        assert!(
            headers.contains("Authorization: Bearer test-key"),
            "request headers: {headers:?}"
        );
        let body: serde_json::Value =
            serde_json::from_slice(&request[header_end + 4..]).expect("chat request JSON");
        assert_eq!(body["model"], "local-model");
        assert_eq!(body["stream"], false);
        assert_eq!(body["temperature"], 0.25);
        assert_eq!(body["max_tokens"], 64.0);
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][1]["content"], "Hello locally");

        let body = r#"{"choices":[{"message":{"role":"assistant","content":"Mock reply"}}]}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .expect("write chat response");
    });

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let cwd = create_temp_dir("openai-compatible-roundtrip");
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        &format!(
            r#"program OpenAiCompatibleRoundtrip;

uses Std.Ai.OpenAi as OpenAi;
uses Std.Console as Console;

begin
  mutable var ClientValue: OpenAi.Client := OpenAi.Client.Create('http://127.0.0.1:{port}/v1', 'local-model');
  ClientValue.ApiKey := Option.Some('test-key');
  ClientValue.TimeoutMillis := 5000;
  mutable var Options: OpenAi.ChatOptions := OpenAi.ChatOptions.Default();
  Options.Temperature := Option.Some(0.25);
  Options.MaxTokens := Option.Some(64);
  case OpenAi.Complete(ClientValue, [
                                      OpenAi.ChatMessage.System('Be concise'),
                                      OpenAi.ChatMessage.User('Hello locally')
                                    ], Options) of
    when Result.Ok(const Content):
      Console.WriteLn(Content);
    when Result.Error(const Message):
      panic(Message);
  end case;
end program;
"#
        ),
    );

    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("run"),
            String::from("--std-lib"),
            root.join("lib").to_string_lossy().into_owned(),
            source.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    std::fs::remove_dir_all(&cwd).expect("temporary directory must be removed");
    assert_eq!(exit, 0, "stderr: {stderr}");
    server.join().expect("OpenAI fixture must finish");
    assert_eq!(stdout, "Mock reply\n");
}
