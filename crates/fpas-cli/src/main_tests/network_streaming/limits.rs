use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[test]
fn imported_facade_stream_is_released_at_block_exit() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind facade fixture");
    let port = listener.local_addr().expect("address").port();
    let server = listener.spawn(move |listener| {
        let (mut stream, _) = listener.accept().expect("accept facade client");
        read_request_head(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello")
            .expect("write response");
    });
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace");
    let cwd = TempDir::new("http-facade-release").expect("fixture directory");
    let project = cwd.join("app.fpasprj");
    write_text(
        &project,
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(
        &cwd.join("facade.fpas"),
        r#"unit App.Facade;
uses Std.Net.Http as Http;
public type Body = Http.BodyStream;
public function Open(Url: string): Body;
begin
  return Http.OpenStream(Http.Request.Get(Url)).Unwrap().Body;
end function;
public function ReadOne(Value: Body): integer;
begin
  return Http.ReadStream(Value, 1).Unwrap().Length();
end function;
public function IsClosed(Value: Body): boolean;
begin
  return Http.ReadStream(Value, 1).Unwrap().Length() = 0;
end function;
end unit;
"#,
    );
    write_text(
        &cwd.join("main.fpas"),
        &format!(
            r#"program FacadeResource;
uses App.Facade as Facade, Std.Test as Test;
begin
  const Original := Facade.Open('http://127.0.0.1:{port}/');
  begin
    using const Body := Original;
    Test.AssertEquals(1, Facade.ReadOne(Body));
  end;
  Test.AssertTrue(Facade.IsClosed(Original));
end.
"#
        ),
    );
    let (exit, _, stderr) = support::run_cli_args_and_capture_output(
        &[
            "run".into(),
            "--std-lib".into(),
            root.join("lib").to_string_lossy().into_owned(),
            project.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    std::fs::remove_dir_all(&cwd).expect("remove fixture");
    assert_eq!(exit, 0, "{stderr}");
    server.join().expect("join fixture");
}

#[test]
fn close_delimited_response_limit_includes_exact_boundary_and_empty_body() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind boundary fixture");
    let port = listener.local_addr().expect("address").port();
    let stopped = Arc::new(AtomicBool::new(false));
    let server_stopped = Arc::clone(&stopped);
    let head = "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n";
    let server = listener.spawn(move |listener| {
        let mut request = 0;
        while !server_stopped.load(Ordering::Acquire) {
            match listener.try_accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_nonblocking(false)
                        .expect("blocking fixture connection");
                    stream
                        .set_read_timeout(Some(Duration::from_secs(5)))
                        .expect("fixture timeout");
                    read_request_head(&mut stream);
                    let body = if request < 6 { "" } else { "abc" };
                    stream
                        .write_all(format!("{head}{body}").as_bytes())
                        .expect("response");
                    request += 1;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("fixture accept: {error}"),
            }
        }
    });
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace");
    let cwd = TempDir::new("http-exact-limit").expect("fixture directory");
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        &format!(
            r#"
program HttpExactLimit;
uses Std.Net.Http as Http, Std.Test as Test;
function Fetch(RequestValue: Http.Request; Streaming: boolean): result of (integer, Http.HttpError);
begin
  if not Streaming then
  begin
    const ResponseValue: Http.Response := try Http.Send(RequestValue);
    return Ok(ResponseValue.Body.Length());
  end; end if;
  const ResponseValue: Http.StreamResponse := try Http.OpenStream(RequestValue);
  using const Body := ResponseValue.Body;
  var Count: integer := 0;
  var Reading: boolean := true;
  while Reading do
  begin
    const Bytes := try Http.ReadStream(Body, 2);
    Count := Count + Bytes.Length();
    Reading := Bytes.Length() > 0;
  end; end while;
  return Ok(Count);
end function;
begin
  for BodyIndex: integer := 0 to 1 do
  begin
    for Streaming: boolean := false to true do
    begin
      for Delta: integer := -1 to 1 do
      begin
        var RequestValue: Http.Request := Http.Request.Get('http://127.0.0.1:{port}/');
        RequestValue.MaxResponseBytes := {head_len} + BodyIndex * 3 + Delta;
        RequestValue.TimeoutMillis := 1000;
        const Received: result of (integer, Http.HttpError) := Fetch(RequestValue, Streaming);
        Test.AssertEquals(Delta >= 0, (Received is Ok(_)));
        if Delta >= 0 then Test.AssertEquals(BodyIndex * 3, Received.Unwrap()); end if;
      end; end for;
    end; end for;
  end; end for;
end.
"#,
            head_len = head.len()
        ),
    );
    let (exit, _, stderr) = support::run_cli_args_and_capture_output(
        &[
            "run".into(),
            "--std-lib".into(),
            root.join("lib").to_string_lossy().into_owned(),
            source.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    stopped.store(true, Ordering::Release);
    std::fs::remove_dir_all(&cwd).expect("remove fixture");
    assert_eq!(exit, 0, "{stderr}");
    server.join().expect("join fixture");
}
