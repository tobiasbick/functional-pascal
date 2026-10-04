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
            "\nprogram HttpExactLimit;\nuses Std.Arrays as Arrays; uses Std.Http as Http; uses Std.Results as Results; uses Std.Test as Test;\nfunction Fetch(RequestValue: Http.Request; Streaming: boolean): result of (integer, string);\nbegin\n  if not Streaming then\n  begin\n    var ResponseValue: Http.Response := try Http.Send(RequestValue);\n    return Result.Ok(Arrays.Length(ResponseValue.Body));\n  end; end if;\n  var ResponseValue: Http.StreamResponse := try Http.OpenStream(RequestValue);\n  mutable var Count: integer := 0;\n  mutable var Reading: boolean := true;\n  while Reading do\n  begin\n    var Bytes: array of (integer) := try Http.ReadStream(ResponseValue.Body, 2);\n    Count := Count + Arrays.Length(Bytes);\n    Reading := Arrays.Length(Bytes) > 0;\n  end; end while;\n  return Result.Ok(Count);\nend function;\nbegin\n  for BodyIndex: integer := 0 to 1 do\n  begin\n    for Streaming: boolean := false to true do\n    begin\n      for Delta: integer := -1 to 1 do\n      begin\n        mutable var RequestValue: Http.Request := Http.Request.Get('http://127.0.0.1:{port}/');\n        RequestValue.MaxResponseBytes := {head_len} + BodyIndex * 3 + Delta;\n        RequestValue.TimeoutMillis := 1000;\n        var Received: result of (integer, string) := Fetch(RequestValue, Streaming);\n        Test.AssertEquals(Delta >= 0, Results.IsOk(Received));\n        if Delta >= 0 then Test.AssertEquals(BodyIndex * 3, Results.Unwrap(Received)); end if;\n      end; end for;\n    end; end for;\n  end; end for;\nend program;\n",
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
