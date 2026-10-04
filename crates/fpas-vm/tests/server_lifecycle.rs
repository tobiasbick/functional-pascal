//! Process escalation is tested only inside disposable child processes.
//! Contract: `docs/pascal/std/network/server.md`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "integration assertions and disposable process fixtures"
)]

use fpas_vm::Vm;
use std::io::{self, Write};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn vm(source: &str) -> Vm {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    Vm::with_writer_and_args(
        fpas_compiler::compile(&program).unwrap(),
        Box::new(io::sink()),
        vec![],
    )
}

#[test]
fn embedding_does_not_implicitly_authorize_process_control() {
    vm(
        r#"program Test;  uses Std.Server as Server; uses Std.Results as Results; uses Std.Tasks as Tasks; uses Std.Test as Test2;
function Require of (T)(Outcome: Result of (T, string)): T;
begin
  case Outcome of
    when Result.Ok(const Value): return Value;
    when Result.Error(const Message): panic(Message);
  end case;
end function;
begin
      Test2.AssertTrue((case Server.CreateLifetime(10, true) of when Result.Ok(_): false; when Result.Error(_): true; end case));
      const Life: Server.ServerLifetime := Require(Server.CreateLifetime(10, false));
      Test2.AssertTrue(Results.IsError(Server.ObserveSignals(Life)));
      discard Server.RequestStop(Life); discard Tasks.CloseTaskGroup(Server.GetWorkGroup(Life));
      Test2.AssertTrue(Results.IsOk(Server.FinishShutdown(Life))); end program;"#,
    )
    .run()
    .unwrap();
}

#[test]
fn stop_rejects_new_group_work() {
    let error = vm(r#"program Test;  uses Std.Server as Server; uses Std.Results as Results; uses Std.Tasks as Tasks;
function Require of (T)(Outcome: Result of (T, string)): T;
begin
  case Outcome of
    when Result.Ok(const Value): return Value;
    when Result.Error(const Message): panic(Message);
  end case;
end function;
begin
      const Life: Server.ServerLifetime := Require(Server.CreateLifetime(10, false));
      discard Server.RequestStop(Life);
      const WorkerHandle: task := Tasks.StartTaskInGroup(Server.GetWorkGroup(Life), function(Token: Tasks.CancellationToken): boolean begin return true; end function);
      end program;"#).run().unwrap_err();
    assert!(
        error.message.contains("closing or cancelled"),
        "{}",
        error.message
    );
}

#[test]
fn returning_without_explicit_cleanup_reports_incomplete_shutdown() {
    let error = vm(
        r#"program Test;  uses Std.Server as Server; uses Std.Results as Results;
function Require of (T)(Outcome: Result of (T, string)): T;
begin
  case Outcome of
    when Result.Ok(const Value): return Value;
    when Result.Error(const Message): panic(Message);
  end case;
end function;
begin
      discard Require(Server.CreateLifetime(0, false)); end program;"#,
    )
    .run()
    .unwrap_err();
    assert!(
        error.message.contains("shutdown incomplete"),
        "{}",
        error.message
    );
}

struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn run_child(mode: &str) -> std::process::ExitStatus {
    let mut child = ChildGuard(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "server_lifecycle_child", "--nocapture"])
            .env("FPAS_SERVER_TEST_MODE", mode)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            assert_ne!(
                status.code(),
                Some(101),
                "child panicked instead of exercising process escalation: {mode}"
            );
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "lifecycle child exceeded hard test timeout: {mode}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn cooperative_completion_disarms_escalation() {
    assert!(run_child("clean").success());
}

#[test]
fn standalone_signal_registration_is_explicit_and_idempotent() {
    assert!(run_child("signals").success());
}

#[test]
fn noncooperative_computation_is_terminated_and_parent_survives() {
    assert!(!run_child("compute").success());
}

#[test]
fn blocked_host_output_cannot_extend_the_deadline() {
    assert!(!run_child("blocked-output").success());
}

#[test]
fn failing_host_output_cannot_extend_the_deadline() {
    assert!(!run_child("failed-output").success());
}

struct BlockedOutput;
impl Write for BlockedOutput {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        loop {
            std::thread::park();
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct FailedOutput;
impl Write for FailedOutput {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::ErrorKind::BrokenPipe.into())
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::ErrorKind::BrokenPipe.into())
    }
}

#[test]
fn server_lifecycle_child() {
    let Ok(mode) = std::env::var("FPAS_SERVER_TEST_MODE") else {
        return;
    };
    let body = if mode == "signals" {
        "Test.AssertTrue(Results.Unwrap(Server.ObserveSignals(Life))); Test.AssertTrue(not Results.Unwrap(Server.ObserveSignals(Life)));
         discard Server.RequestStop(Life); discard Tasks.CloseTaskGroup(Server.GetWorkGroup(Life)); Test.AssertTrue(Results.IsOk(Server.FinishShutdown(Life)));"
    } else if mode == "clean" {
        "discard Server.RequestStop(Life); discard Tasks.CloseTaskGroup(Server.GetWorkGroup(Life));
         Test.AssertTrue(Results.IsOk(Server.FinishShutdown(Life))); Time.Sleep(200);"
    } else if mode == "compute" {
        "discard Server.RequestStop(Life); while true do null; end while;"
    } else {
        "discard Server.RequestStop(Life); Console.WriteLn('output');"
    };
    let source = format!(
        r#"program Child;  uses Std.Server as Server; uses Std.Results as Results; uses Std.Tasks as Tasks; uses Std.Time as Time; uses Std.Console as Console; uses Std.Test as Test;
function Require of (T)(Outcome: Result of (T, string)): T;
begin
  case Outcome of
    when Result.Ok(const Value): return Value;
    when Result.Error(const Message): panic(Message);
  end case;
end function;
begin const Life: Server.ServerLifetime := Require(Server.CreateLifetime(50, true)); {body} end program;"#
    );
    let (program, errors) = fpas_parser::parse(&source);
    assert!(errors.is_empty(), "{errors:?}");
    let output: Box<dyn Write + Send> = match mode.as_str() {
        "blocked-output" => Box::new(BlockedOutput),
        "failed-output" => Box::new(FailedOutput),
        _ => Box::new(io::sink()),
    };
    let mut vm =
        Vm::with_writer_and_args(fpas_compiler::compile(&program).unwrap(), output, vec![]);
    vm.allow_process_lifecycle();
    let result = vm.run();
    if mode == "clean" || mode == "signals" {
        result.unwrap();
    } else {
        // A host diagnostic must not disarm the deadline before the process has exited.
        // Keep the VM alive to model an embedding host blocked in diagnostic handling.
        std::thread::sleep(Duration::from_secs(5));
        panic!("explicit escalation did not terminate the child");
    }
}
