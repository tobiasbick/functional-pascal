//! Public terminal providers exercise independent child input, output and owned cleanup.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "process fixture assertions"
)]

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

use fpas_runtime::execution::{
    DebugRunResult, DebugSession, InteractiveProcessStdio, InteractiveProcessTerminal, Vm,
};

#[cfg(target_os = "linux")]
#[path = "process_terminal/foreground.rs"]
mod foreground;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "fpas-terminal-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("input"), "interactive input\n").unwrap();
        Self(path)
    }

    fn terminal(&self) -> InteractiveProcessTerminal {
        let root = self.0.clone();
        InteractiveProcessTerminal::new(move || {
            Ok(InteractiveProcessStdio::new(
                File::open(root.join("input"))?.into(),
                File::create(root.join("output"))?.into(),
                File::create(root.join("error"))?.into(),
            ))
        })
    }

    fn vm(&self, body: &str, environment: &str) -> Vm {
        let source = self.source(body, environment);
        Vm::with_writer_and_args(
            compile(&source),
            Box::new(io::sink()),
            vec![
                std::env::current_exe().unwrap().to_string_lossy().into(),
                self.0.to_string_lossy().into(),
            ],
        )
    }

    fn source(&self, body: &str, environment: &str) -> String {
        format!(
            r#"program Terminal;
uses Std.Proc as Proc, Std.Args as Args, Std.Test as Test, Std.Tasks as Tasks, Std.Fs as Fs, Std.Time as Time;
procedure ActWhenReady(Source: Tasks.CancellationSource; Marker: string; Fail: boolean);
begin
  const Deadline := Time.MonotonicMillis() + 15000;
  while not Fs.Exists(Marker) do
    Test.AssertTrue(Time.MonotonicMillis() < Deadline);
    Time.Sleep(1);
  end while;
  if Fail then panic('intentional terminal scope failure'); end if;
  discard Tasks.Cancel(Source);
end procedure;
begin
  const Options := Proc.DefaultStartOptions() with
    WorkingDirectory := Some(Args.ParamStr(1));
    Environment := [Proc.EnvironmentVariable(Name := 'FPAS_TERMINAL_VALUE', Value := Some('child value')){environment}];
  end with;
  {body}
end."#
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn compile(source: &str) -> fpas_runtime::VerifiedExecutable {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program).expect("compile terminal caller")
}

const RUN: &str = "Proc.RunInteractive(Args.ParamStr(0), ['--ignored', '--exact', 'terminal_child', '--nocapture'], Options, Some(15000), None)";

#[test]
fn configured_streams_and_exact_status_work_in_vm_and_explicit_debugger_host() {
    let fixture = Fixture::new();
    let body = format!(
        "const Status := {RUN}.Unwrap(); Test.AssertEquals(7, Status.ExitCode.Unwrap()); Test.AssertTrue(Status.Signal is None);"
    );
    let parent_cwd = std::env::current_dir().unwrap();
    let parent_env = std::env::var_os("FPAS_TERMINAL_VALUE");
    let mut vm = fixture.vm(&body, "");
    vm.set_interactive_process_terminal(fixture.terminal());
    vm.run().expect("interactive VM caller");
    let output = fs::read_to_string(fixture.0.join("output")).unwrap();
    assert!(output.contains("INPUT:interactive input"), "{output}");
    assert!(output.contains("VALUE:child value"), "{output}");
    assert!(
        output.contains(&format!("CWD:{}", fixture.0.display())),
        "{output}"
    );
    assert_eq!(
        fs::read_to_string(fixture.0.join("error")).unwrap(),
        "terminal error\n"
    );
    assert_eq!(std::env::current_dir().unwrap(), parent_cwd);
    assert_eq!(std::env::var_os("FPAS_TERMINAL_VALUE"), parent_env);

    let args = vec![
        std::env::current_exe().unwrap().to_string_lossy().into(),
        fixture.0.to_string_lossy().into(),
    ];
    let mut debug = DebugSession::with_args(compile(&fixture.source(&body, "")), args).unwrap();
    debug
        .set_interactive_process_terminal(fixture.terminal())
        .unwrap();
    assert!(matches!(
        debug.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn missing_terminal_and_invalid_or_exhausted_budgets_launch_nothing() {
    let fixture = Fixture::new();
    fixture
        .vm(
            &format!(
                "Test.AssertTrue({RUN} is Error(Proc.ProcError.InvalidArgument('Terminal', _)));"
            ),
            "",
        )
        .run()
        .unwrap();
    let body = r#"
      const Source := Tasks.CreateCancellationSource();
      discard Tasks.Cancel(Source);
      const Token := Some(Tasks.GetCancellationToken(Source));
      Test.AssertTrue(Proc.RunInteractive('unused', [], Options, Some(-1), Token) is Error(Proc.ProcError.InvalidArgument('TimeoutMillis', _)));
      Test.AssertTrue(Proc.RunInteractive('unused', [], Options, Some(0), Token) is Error(Proc.ProcError.Cancelled(_, '', '')));
      Test.AssertTrue(Proc.RunInteractive('unused', [], Options, Some(0), None) is Error(Proc.ProcError.Timeout(_, '', '')));
    "#;
    let opened = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&opened);
    let mut vm = fixture.vm(body, "");
    vm.set_interactive_process_terminal(InteractiveProcessTerminal::new(move || {
        count.fetch_add(1, Ordering::Relaxed);
        Err(io::Error::other("must not open terminal"))
    }));
    vm.run().unwrap();
    assert_eq!(opened.load(Ordering::Relaxed), 0);

    let source = fixture.source(
        &format!("Test.AssertTrue({RUN} is Error(Proc.ProcError.InvalidArgument('Terminal', _)));"),
        "",
    );
    let args = vec![
        std::env::current_exe().unwrap().to_string_lossy().into(),
        fixture.0.to_string_lossy().into(),
    ];
    let mut debug = DebugSession::with_args(compile(&source), args).unwrap();
    debug.push_debuggee_input("protocol-owned input").unwrap();
    assert!(matches!(
        debug.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
    assert!(!fixture.0.join("output").exists());
}

#[test]
fn aborted_interactive_calls_reap_descendants_and_preserve_scope_failures() {
    for mode in ["cancel", "timeout", "shutdown", "failure"] {
        let fixture = Fixture::new();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let environment = format!(
            ", Proc.EnvironmentVariable(Name := 'FPAS_TERMINAL_ADDRESS', Value := Some('{}'))",
            listener.local_addr().unwrap()
        );
        let body = match mode {
            "cancel" => r#"scope
                const Source := Tasks.CreateCancellationSource();
                const Canceller: task := go ActWhenReady(Source, Args.ParamStr(1) + '/cancel', false);
                const Outcome := Proc.RunInteractive(Args.ParamStr(0), ['--ignored', '--exact', 'terminal_child', '--nocapture'], Options, Some(15000), Some(Tasks.GetCancellationToken(Source)));
                Test.AssertTrue(Outcome is Error(Proc.ProcError.Cancelled(_, '', '')));
                Tasks.Wait(Canceller);
              end scope;"#.into(),
            "timeout" => format!("Test.AssertTrue({} is Error(Proc.ProcError.Timeout(_, '', '')));", RUN.replace("Some(15000)", "Some(2000)")),
            "failure" => r#"scope
                const Source := Tasks.CreateCancellationSource();
                const Failer: task := go ActWhenReady(Source, Args.ParamStr(1) + '/cancel', true);
                discard Proc.RunInteractive(Args.ParamStr(0), ['--ignored', '--exact', 'terminal_child', '--nocapture'], Options, None, None);
                Tasks.Wait(Failer);
              end scope;"#.into(),
            _ => format!("discard {RUN};"),
        };
        let mut vm = fixture.vm(&body, &environment);
        vm.set_interactive_process_terminal(fixture.terminal());
        let shutdown = vm.shutdown_handle();
        let worker = std::thread::spawn(move || vm.run());
        let mut receipts = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(20);
        while receipts.len() < 2 && Instant::now() < deadline {
            match listener.accept() {
                Ok((stream, _)) => receipts.push(stream),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(1))
                }
                Err(error) => panic!("child receipt: {error}"),
            }
        }
        if mode == "cancel" || mode == "failure" {
            fs::write(fixture.0.join("cancel"), "cancel").unwrap();
        }
        if mode == "shutdown" || receipts.len() != 2 {
            shutdown.shutdown();
        }
        let (finished, completion) = std::sync::mpsc::channel::<()>();
        let stop = shutdown.clone();
        let watchdog = std::thread::spawn(move || {
            if completion.recv_timeout(Duration::from_secs(10)).is_err() {
                stop.shutdown();
            }
        });
        let result = worker.join().unwrap();
        let _ = finished.send(());
        watchdog.join().unwrap();
        assert_eq!(
            receipts.len(),
            2,
            "{mode}: both leader and descendant must start; VM: {result:?}; child stderr: {}",
            fs::read_to_string(fixture.0.join("error")).unwrap_or_default()
        );
        if mode == "failure" {
            let failure =
                result.expect_err("scope failure propagates after interrupting the host call");
            assert!(
                failure
                    .message
                    .contains("intentional terminal scope failure"),
                "{failure:?}"
            );
        } else if mode != "shutdown" {
            result.expect(mode);
        }
        for mut stream in receipts {
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            match stream.read(&mut [0]) {
                Ok(0) => {}
                Err(error) if error.kind() == io::ErrorKind::ConnectionReset => {}
                result => panic!("{mode}: descendant survived cleanup: {result:?}"),
            }
        }
    }
}

#[test]
fn precise_status_retains_platform_completion_without_consuming_legacy_status() {
    let fixture = Fixture::new();
    #[cfg(windows)]
    let (native, expected, legacy) = (
        "4294967295",
        "Test.AssertEquals(4294967295, Status.ExitCode.Unwrap()); Test.AssertTrue(Status.Signal is None);",
        -1,
    );
    #[cfg(unix)]
    let (native, expected, legacy) = (
        "signal",
        "Test.AssertTrue(Status.ExitCode is None); Test.AssertEquals('SIGTERM', Status.Signal.Unwrap());",
        -1,
    );
    let environment = format!(
        ", Proc.EnvironmentVariable(Name := 'FPAS_TERMINAL_EXIT', Value := Some('{native}'))"
    );
    let body = format!(
        r#"
      const Child := Proc.Start(Args.ParamStr(0), ['--ignored', '--exact', 'terminal_child', '--nocapture'], Options).Unwrap();
      const Deadline := Time.MonotonicMillis() + 15000;
      while Proc.TryWaitStatus(Child).Unwrap() is None do
        Test.AssertTrue(Time.MonotonicMillis() < Deadline);
        Time.Sleep(1);
      end while;
      const Status := Proc.TryWaitStatus(Child).Unwrap().Unwrap();
      {expected}
      Test.AssertEquals({legacy}, Proc.TryWait(Child).Unwrap().Unwrap());
      const Again := Proc.TryWaitStatus(Child).Unwrap().Unwrap();
      Test.AssertTrue(Status.ExitCode = Again.ExitCode);
      Test.AssertTrue(Status.Signal = Again.Signal);
      discard Proc.Close(Child).Unwrap();
    "#
    );
    fixture
        .vm(&body, &environment)
        .run()
        .expect("precise cached status");
}

#[test]
#[ignore = "independent interactive child fixture"]
#[expect(
    clippy::zombie_processes,
    reason = "The fixture deliberately leaves a descendant for the owning process-group cleanup test"
)]
fn terminal_child() {
    if let Ok(status) = std::env::var("FPAS_TERMINAL_EXIT") {
        #[cfg(unix)]
        if status == "signal" {
            nix::sys::signal::raise(nix::sys::signal::Signal::SIGTERM).unwrap();
            panic!("SIGTERM must terminate the child");
        }
        std::process::exit(status.parse::<u32>().unwrap() as i32);
    }
    if let Ok(address) = std::env::var("FPAS_TERMINAL_ADDRESS") {
        let _child = Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "terminal_descendant", "--nocapture"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut stream = TcpStream::connect(address).unwrap();
        let _ = stream.read(&mut [0]);
        return;
    }
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    println!("INPUT:{}", input.trim_end());
    println!("VALUE:{}", std::env::var("FPAS_TERMINAL_VALUE").unwrap());
    println!("CWD:{}", std::env::current_dir().unwrap().display());
    io::stderr().write_all(b"terminal error\n").unwrap();
    std::process::exit(7);
}

#[test]
#[ignore = "independent descendant liveness fixture"]
fn terminal_descendant() {
    let mut stream = TcpStream::connect(std::env::var("FPAS_TERMINAL_ADDRESS").unwrap()).unwrap();
    let _ = stream.read(&mut [0]);
}
