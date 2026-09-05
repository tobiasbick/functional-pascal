use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;

use super::*;

mod limits;

fn read_request_head(stream: &mut TcpStream) {
    let mut request = Vec::new();
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        let mut buffer = [0_u8; 1024];
        let count = stream.read(&mut buffer).expect("read streaming request");
        assert!(count > 0, "streaming request ended before its headers");
        request.extend_from_slice(&buffer[..count]);
    }
}

fn write_chunk(stream: &mut TcpStream, bytes: &[u8]) {
    write!(stream, "{:x}\r\n", bytes.len()).expect("write chunk size");
    stream.write_all(bytes).expect("write chunk body");
    stream.write_all(b"\r\n").expect("write chunk terminator");
    stream.flush().expect("flush response chunk");
}

#[test]
fn http_stream_decodes_fragmented_chunked_sse_response() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind streaming fixture");
    let port = listener
        .local_addr()
        .expect("streaming fixture address")
        .port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept streaming client");
        read_request_head(&mut stream);
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
            )
            .expect("write streaming response head");
        stream.flush().expect("flush streaming response head");
        std::thread::sleep(Duration::from_millis(20));
        write_chunk(&mut stream, b"event: token\ndata: hel");
        std::thread::sleep(Duration::from_millis(20));
        write_chunk(&mut stream, b"lo\n\ndata: wor");
        std::thread::sleep(Duration::from_millis(20));
        write_chunk(&mut stream, b"ld\n\n");
        stream
            .write_all(b"0\r\n\r\n")
            .expect("write final response chunk");
    });

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let cwd = create_temp_dir("http-streaming-roundtrip");
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        &format!(
            r#"program HttpStreamingRoundtrip;

uses Std.Array, Std.Console, Std.Http;

procedure PrintEvents(Events: array of SseEvent);
begin
  for Index: integer := 0 to Std.Array.Length(Events) - 1 do
  begin
    WriteLn((Events[Index].EventType + ':') + Events[Index].Data)
  end
end;

begin
  case OpenStream(Request.Get('http://127.0.0.1:{port}/events')) of
    Ok(ResponseValue):
    begin
      WriteLn(ResponseValue.StatusCode);
      case CreateSseDecoder(4096) of
        Ok(Decoder):
        begin
          mutable var Reading: boolean := true;
          while Reading do
          begin
            case ReadStream(ResponseValue.Body, 3) of
              Ok(Bytes):
              begin
                if Std.Array.Length(Bytes) = 0 then
                begin
                  Reading := false
                end
                else
                begin
                  case FeedSse(Decoder, Bytes) of
                    Ok(Events): PrintEvents(Events);
                    Error(Message): panic(Message)
                  end
                end
              end;
              Error(Message): panic(Message)
            end
          end;
          case FinishSse(Decoder) of
            Ok(Events): PrintEvents(Events);
            Error(Message): panic(Message)
          end
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
    server.join().expect("streaming fixture must finish");

    assert_eq!(exit, 0, "stderr: {stderr}");
    assert_eq!(stdout, "200\ntoken:hello\nmessage:world\n");
}

#[test]
fn http_stream_rejects_truncated_content_length() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind truncated response fixture");
    let port = listener
        .local_addr()
        .expect("truncated fixture address")
        .port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept truncated response client");
        read_request_head(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nabc")
            .expect("write truncated response");
    });

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let cwd = create_temp_dir("http-streaming-truncated");
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        &format!(
            r#"program HttpStreamingTruncated;

uses Std.Array, Std.Console, Std.Http, Std.Str;

begin
  case OpenStream(Request.Get('http://127.0.0.1:{port}/truncated')) of
    Ok(ResponseValue):
    begin
      case ReadStream(ResponseValue.Body, 8) of
        Ok(Bytes):
        begin
          if Std.Array.Length(Bytes) <> 3 then
          begin
            panic('unexpected first body fragment')
          end
        end;
        Error(Message): panic(Message)
      end;
      case ReadStream(ResponseValue.Body, 8) of
        Ok(_): panic('truncated Content-Length was accepted');
        Error(Message):
        begin
          if not Std.Str.Contains(Message, 'shorter than Content-Length') then
          begin
            panic(Message)
          end
        end
      end;
      WriteLn('ok')
    end;
    Error(Message): panic(Message)
  end
end.
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
    server.join().expect("truncated fixture must finish");

    assert_eq!(exit, 0, "stderr: {stderr}");
    assert_eq!(stdout, "ok\n");
}
