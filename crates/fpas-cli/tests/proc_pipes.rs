//! Real child-process exchanges verify raw pipes before exit, EOF and VM ownership.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration fixture failures need context"
)]

use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::process::Stdio;
use std::time::{Duration, Instant};
use std::{env, fs};

#[path = "../test_support/process.rs"]
mod process;

struct DapProcess {
    child: process::Process,
    reader: Option<std::thread::JoinHandle<()>>,
}

impl Drop for DapProcess {
    fn drop(&mut self) {
        let _ = self.child.terminate();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[test]
fn dap_terminal_acquisition_reaches_a_breakpoint_without_keepalive_requests() {
    let root = process::TempDir::new("fpas-dap-terminal").expect("fixture directory");
    let source = root.join("terminal.fpas");
    fs::write(&source, "program TerminalProbe; uses Std.Console as Console; procedure Marker(); begin Console.WriteLn('ready'); end procedure; begin Console.AcquireInteractiveTerminal(); Marker(); Console.ReleaseInteractiveTerminal(); end.").expect("terminal probe");
    let mut command = Command::new(env!("CARGO_BIN_EXE_fpas"));
    command
        .args(["debug"])
        .arg(&source)
        .args(["--protocol", "dap"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child =
        process::Process::spawn_raw(command, Duration::from_secs(10)).expect("DAP process");
    let mut input = child.take_stdin().expect("DAP input");
    let output = child.take_stdout().expect("DAP output");
    let (sender, receiver) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut output = BufReader::new(output);
        loop {
            let mut header = String::new();
            if output.read_line(&mut header).unwrap_or(0) == 0 {
                break;
            }
            let Some(length) = header
                .trim()
                .strip_prefix("Content-Length: ")
                .and_then(|s| s.parse::<usize>().ok())
            else {
                break;
            };
            let mut empty = String::new();
            if output.read_line(&mut empty).is_err() {
                break;
            }
            let mut body = vec![0; length];
            if output.read_exact(&mut body).is_err() {
                break;
            }
            let message: serde_json::Value = serde_json::from_slice(&body).expect("DAP JSON");
            if sender.send(message).is_err() {
                break;
            }
        }
    });
    let mut process = DapProcess {
        child,
        reader: Some(reader),
    };
    for (index, (command, arguments)) in [
        ("initialize", serde_json::json!({})),
        ("launch", serde_json::json!({"stopOnEntry": false})),
        (
            "setFunctionBreakpoints",
            serde_json::json!({"breakpoints": [{"name":"Marker"}]}),
        ),
        ("configurationDone", serde_json::json!({})),
    ]
    .into_iter()
    .enumerate()
    {
        let body = serde_json::json!({"seq":index + 1,"type":"request","command":command,"arguments":arguments}).to_string();
        write!(input, "Content-Length: {}\r\n\r\n{body}", body.len()).expect("DAP request");
        input.flush().expect("request flush");
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut stopped = false;
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        match receiver.recv_timeout(remaining) {
            Ok(message) if message["event"] == "stopped" => {
                stopped = message["body"]["reason"] == "breakpoint";
                break;
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    process.child.terminate().expect("DAP process cleanup");
    drop(input);
    process
        .reader
        .take()
        .expect("DAP reader")
        .join()
        .expect("DAP reader cleanup");
    assert!(
        stopped,
        "terminal acquisition blocked with the protocol reader waiting for input"
    );
}

#[test]
#[ignore = "subprocess fixture for process ownership checks"]
fn owned_child() {
    let _liveness = TcpStream::connect(env::var("FPAS_PROCESS_ADDRESS").expect("address"))
        .expect("liveness connection");
    std::io::stdout().write_all(b"ready").expect("ready");
    std::io::stdout().flush().expect("flush");
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).expect("stdin");
    std::thread::sleep(Duration::from_secs(30));
}

#[test]
#[ignore = "subprocess fixture launched by interactive_pipes_exchange_raw_bytes_and_close_aliases"]
fn pipe_child() {
    std::io::stdout()
        .write_all(b"\0\xffready")
        .expect("raw readiness");
    std::io::stdout().flush().expect("stdout flush");
    std::io::stderr()
        .write_all(b"\0\xfefault")
        .expect("raw stderr");
    std::io::stderr().flush().expect("stderr flush");
    let mut input = [0; 4];
    std::io::stdin()
        .read_exact(&mut input)
        .expect("interactive input");
    assert_eq!(input, [0, 255, 195, 164]);
    std::io::stdout()
        .write_all(b"\0reply")
        .expect("unterminated reply");
    std::io::stdout().flush().expect("reply flush");
    let mut remaining = Vec::new();
    std::io::stdin()
        .read_to_end(&mut remaining)
        .expect("stdin EOF");
    assert!(remaining.is_empty());
}

const SOURCE: &str = r#"// Exercises interactive binary pipes against a live child.
program Pipes;
uses Std.Os.Args as Args, Std.Os.Proc as Proc, Std.Test as Test, Std.Time as Time, Std.Os.Errors as Errors;

function ReadUntil(Child: Proc.Process; Needle: array of integer; Stderr: boolean): array of integer;
begin
  var Bytes: array of integer := [];
  const Deadline := Time.MonotonicMillis() + 15000;
  while Time.MonotonicMillis() < Deadline do
    const Chunk := if Stderr then Proc.ReadStderr(Child, 3).Unwrap() else Proc.ReadStdout(Child, 3).Unwrap() end if;
    case Chunk of
      when Some(const More):
        Test.AssertTrue(not More.IsEmpty());
        Bytes := Bytes.Concat(More);
        for I: integer := 0 to Bytes.Length() - Needle.Length() do
          var Matches := true;
          for J: integer := 0 to Needle.Length() - 1 do
            Matches := Matches and (Bytes[I + J] = Needle[J]);
          end for;
          if Matches then return Bytes; end if;
        end for;
      when None: Time.Sleep(1);
    end case;
  end while;
  panic('child pipe timed out');
end function;

begin
  const Options := Proc.DefaultStartOptions();
  Test.AssertTrue(Options.WorkingDirectory is None);
  Test.AssertTrue(Options.Environment.IsEmpty());
  case Proc.Start('__fpas_missing_interactive_command__', [], Options) of
    when Error(Proc.ProcError.StartFailed(Errors.HostError.Failure(const Operation, const Resources, _, _, _))):
      Test.AssertEquals('Std.Os.Proc.Start', Operation);
      Test.AssertEquals(1, Resources.Length());
      Test.AssertEquals('__fpas_missing_interactive_command__', Resources[0]);
    when Ok(_), Error(_):
      panic('expected a typed native launch failure');
  end case;
  const Child := Proc.Start(Args.ParamStr(0), ['--ignored', '--exact', 'pipe_child', '--nocapture'], Options).Unwrap();
  const Alias := Child;
  discard ReadUntil(Child, [0, 255, 114, 101, 97, 100, 121], false);
  discard ReadUntil(Child, [0, 254, 102, 97, 117, 108, 116], true);
  Test.AssertTrue(Proc.TryWait(Child).Unwrap() is None);
  Test.AssertTrue(Proc.TryWaitStatus(Child).Unwrap() is None);
  Test.AssertTrue(Proc.ReadStdout(Child, 2).Unwrap() is None);
  Test.AssertTrue(Proc.WriteStdin(Child, [256]) is Error(_));
  Test.AssertTrue(Proc.WriteStdin(Child, [-1]) is Error(_));
  Test.AssertTrue(Proc.ReadStdout(Child, 0) is Error(_));
  Test.AssertTrue(Proc.ReadStderr(Child, 1048577) is Error(_));
  var Sent := 0;
  const Input := [0, 255, 195, 164];
  while Sent < Input.Length() do
    Sent := Sent + Proc.WriteStdin(Child, Input.Slice(Sent, Input.Length() - Sent)).Unwrap();
    Time.Sleep(1);
  end while;
  discard ReadUntil(Child, [0, 114, 101, 112, 108, 121], false);
  Test.AssertTrue(Proc.TryWait(Child).Unwrap() is None);
  while not Proc.CloseStdin(Child).Unwrap() do Time.Sleep(1); end while;
  Test.AssertTrue(Proc.CloseStdin(Child).Unwrap());
  Test.AssertTrue(Proc.WriteStdin(Child, [1]) is Error(_));
  const Deadline := Time.MonotonicMillis() + 15000;
  var Done := false;
  while not Done and (Time.MonotonicMillis() < Deadline) do
    case Proc.ReadStdout(Child, 4096).Unwrap() of
      when Some(const Bytes): Done := Bytes.IsEmpty();
      when None: Time.Sleep(1);
    end case;
  end while;
  Test.AssertTrue(Done);
  while Proc.TryWait(Child).Unwrap() is None do
    Test.AssertTrue(Time.MonotonicMillis() < Deadline);
    Time.Sleep(1);
  end while;
  Test.AssertEquals(0, Proc.TryWait(Child).Unwrap().Unwrap());
  const Status := Proc.TryWaitStatus(Alias).Unwrap().Unwrap();
  Test.AssertEquals(0, Status.ExitCode.Unwrap());
  Test.AssertTrue(Status.Signal is None);
  const Again := Proc.TryWaitStatus(Child).Unwrap().Unwrap();
  Test.AssertTrue(Status.ExitCode = Again.ExitCode);
  Test.AssertTrue(Status.Signal = Again.Signal);
  var StderrEof := false;
  while not StderrEof and (Time.MonotonicMillis() < Deadline) do
    case Proc.ReadStderr(Child, 4096).Unwrap() of
      when Some(const Bytes): StderrEof := Bytes.IsEmpty();
      when None: Time.Sleep(1);
    end case;
  end while;
  Test.AssertTrue(StderrEof);
  Test.AssertTrue(Proc.ReadStdout(Child, 4096).Unwrap().Unwrap().IsEmpty());
  discard Proc.Close(Child).Unwrap();
  Test.AssertTrue(Proc.TryWait(Alias) is Error(_));
  Test.AssertTrue(Proc.TryWaitStatus(Alias) is Error(_));
  Test.AssertTrue(Proc.Close(Alias) is Error(_));
  Test.AssertTrue(Proc.Start('', [], Options) is Error(_));
end.
"#;

#[test]
fn termination_close_completion_and_failure_reap_interactive_children() {
    let root = process::TempDir::new("fpas-pipe-owner").expect("fixture directory");
    let source = root.join("owner.fpas");
    let prefix = SOURCE
        .split("begin\n  const Options :=")
        .next()
        .expect("shared controller helpers");
    fs::write(&source, prefix.to_owned() + r#"begin
  const Options := Proc.DefaultStartOptions() with
    WorkingDirectory := Some('.');
    Environment := [Proc.EnvironmentVariable(Name := 'FPAS_PROCESS_ADDRESS', Value := Some(Args.ParamStr(1)))];
  end with;
  const Child := Proc.Start(Args.ParamStr(0), ['--ignored', '--exact', 'owned_child', '--nocapture'], Options).Unwrap();
  discard ReadUntil(Child, [114, 101, 97, 100, 121], false);
  case Args.ParamStr(2) of
    when 'terminate':
      discard Proc.Terminate(Child).Unwrap();
      const Deadline := Time.MonotonicMillis() + 15000;
      while Proc.TryWait(Child).Unwrap() is None do
        Test.AssertTrue(Time.MonotonicMillis() < Deadline);
        Time.Sleep(1);
      end while;
      discard Proc.Close(Child).Unwrap();
    when 'close': discard Proc.Close(Child).Unwrap();
    when 'return': discard 0;
    when 'panic': panic('intentional ownership failure');
    else panic('invalid mode');
  end case;
end.
"#).expect("owner controller");
    for mode in ["terminate", "close", "return", "panic"] {
        let listener = TcpListener::bind("127.0.0.1:0").expect("liveness listener");
        let mut command = Command::new(env!("CARGO_BIN_EXE_fpas"));
        command
            .current_dir(&root)
            .arg("run")
            .arg(&source)
            .arg("--")
            .arg(env::current_exe().expect("test fixture executable"))
            .arg(listener.local_addr().expect("address").to_string())
            .arg(mode);
        let mut child =
            process::Process::spawn(command, Duration::from_secs(30)).expect("bounded FPAS owner");
        child
            .observe_exit()
            .expect("FPAS owner exit before fallback cleanup");
        listener
            .set_nonblocking(true)
            .expect("nonblocking listener");
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    stream
                        .set_nonblocking(false)
                        .expect("blocking liveness stream");
                    break stream;
                }
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        && Instant::now() < deadline =>
                {
                    std::thread::sleep(Duration::from_millis(1))
                }
                Err(error) => panic!("child never connected: {error}\n{}", child.diagnostics()),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .expect("liveness timeout");
        match stream.read(&mut [0]) {
            Ok(0) => {}
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
            other => panic!("owned child must have exited after {mode}: {other:?}"),
        }
        let output = child
            .wait()
            .expect("owner output after SUT child-liveness assertion");
        assert_eq!(
            output.status.success(),
            mode != "panic",
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn interactive_pipes_exchange_raw_bytes_and_close_aliases() {
    let root = process::TempDir::new("fpas-pipes").expect("fixture directory");
    let source = root.join("pipes.fpas");
    fs::write(&source, SOURCE).expect("FPAS controller");
    let project = root.join("pipes.fpasprj");
    fs::write(&project, "[project]\nname = 'pipes'\nkind = 'program'\nmain = 'pipes.fpas'\n[sources]\ninclude = ['pipes.fpas']\n").expect("controller project");
    let child = env::current_exe().expect("fixture executable");
    let mut command = Command::new(env!("CARGO_BIN_EXE_fpas"));
    command.current_dir(&root).arg("build").arg(&project);
    let build = process::capture(command, Duration::from_secs(30)).expect("build controller image");
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    for artifact in [false, true] {
        let target = if artifact {
            root.join("pipes.fpascp")
        } else {
            source.clone()
        };
        let mut command = Command::new(env!("CARGO_BIN_EXE_fpas"));
        command
            .current_dir(&root)
            .arg("run")
            .arg(target)
            .arg("--")
            .arg(&child);
        let output =
            process::capture(command, Duration::from_secs(30)).expect("bounded controller process");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
