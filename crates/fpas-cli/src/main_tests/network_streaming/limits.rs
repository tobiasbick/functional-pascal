use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[test]
fn close_delimited_response_limit_includes_exact_boundary_and_empty_body() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind boundary fixture");
    let port = listener.local_addr().expect("address").port();
    listener.set_nonblocking(true).expect("nonblocking fixture");
    let stopped = Arc::new(AtomicBool::new(false));
    let server_stopped = Arc::clone(&stopped);
    let head = "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n";
    let server = std::thread::spawn(move || {
        let mut request = 0;
        while !server_stopped.load(Ordering::Acquire) {
            match listener.accept() {
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
    let cwd = create_temp_dir("http-exact-limit");
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        &format!(
            r#"
program HttpExactLimit;
uses Std.Array, Std.Http, Std.Result, Std.Test;
function Fetch(RequestValue: Request; Streaming: boolean): result of integer, string;
begin
  if not Streaming then
  begin
    var ResponseValue: Response := try Send(RequestValue);
    return Ok(Std.Array.Length(ResponseValue.Body))
  end;
  var ResponseValue: StreamResponse := try OpenStream(RequestValue);
  mutable var Count: integer := 0;
  mutable var Reading: boolean := true;
  while Reading do
  begin
    var Bytes: array of integer := try ReadStream(ResponseValue.Body, 2);
    Count := Count + Std.Array.Length(Bytes);
    Reading := Std.Array.Length(Bytes) > 0
  end;
  return Ok(Count)
end;
begin
  for BodyIndex: integer := 0 to 1 do
  begin
    for Streaming: boolean := false to true do
    begin
      for Delta: integer := -1 to 1 do
      begin
        mutable var RequestValue: Request := Request.Get('http://127.0.0.1:{port}/');
        RequestValue.MaxResponseBytes := {head_len} + BodyIndex * 3 + Delta;
        RequestValue.TimeoutMillis := 1000;
        var Received: result of integer, string := Fetch(RequestValue, Streaming);
        AssertEquals(Delta >= 0, Std.Result.IsOk(Received));
        if Delta >= 0 then AssertEquals(BodyIndex * 3, Std.Result.Unwrap(Received))
      end
    end
  end
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
    server.join().expect("join fixture");
    std::fs::remove_dir_all(&cwd).expect("remove fixture");
    assert_eq!(exit, 0, "{stderr}");
}
