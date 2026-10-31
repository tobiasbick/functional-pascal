//! Resource release through the verified bytecode execution boundary.

use super::calls::{FunctionSpec, abc, image};
use super::support::abx;
use crate::execution::vm::Vm;
use crate::{Constant, Opcode, ReturnConvention};
use std::io::{self, Write};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

struct Capture {
    bytes: Arc<Mutex<Vec<u8>>>,
    ready: mpsc::Sender<()>,
}

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut saved = self.bytes.lock().unwrap();
        saved.extend_from_slice(bytes);
        if saved
            .windows(b"parent-ready".len())
            .any(|part| part == b"parent-ready")
        {
            let _ = self.ready.send(());
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn failing_release(body_panics: bool) -> crate::VerifiedExecutable {
    let exit = if body_panics {
        abc(Opcode::Panic, 0, 0, 0, 0)
    } else {
        abc(Opcode::ExitCleanupScope, crate::NO_REGISTER, 0, 0, 0)
    };
    image(
        vec![
            abc(Opcode::EnterCleanupScope, 0, 0, 0, 0),
            abc(Opcode::BeginResourceInit, 0, 0, 0, 0),
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeClosure, 1, 1, 0, 0),
            abc(Opcode::RegisterCleanup, 0, 1, 0, 0),
            exit,
            abc(Opcode::Return, crate::NO_REGISTER, 0, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::Panic, 1, 0, 0, 0),
            abc(Opcode::Return, crate::NO_REGISTER, 0, 0, 0),
        ],
        vec![Constant::Integer(1), Constant::Integer(2)],
        &[
            FunctionSpec {
                start: 0,
                end: 7,
                arity: 0,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 7,
                end: 10,
                arity: 1,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Unit,
            },
        ],
    )
}

#[test]
fn release_failure_overrides_normal_exit() {
    let error = Vm::new(failing_release(false))
        .run()
        .expect_err("release must run");
    assert!(error.message.contains("panic: 2"), "{error:?}");
}

#[test]
fn body_panic_survives_release_failure() {
    let error = Vm::new(failing_release(true))
        .run()
        .expect_err("body must fail");
    assert!(error.message.contains("panic: 1"), "{error:?}");
}

#[test]
fn release_failure_overrides_scope_and_vm_cancellation() {
    use crate::execution::vm::{dispatch::DispatchStep, tasks::TaskScheduler, worker::Worker};

    for vm_shutdown in [false, true] {
        let scheduler = Arc::new(TaskScheduler::new());
        let (scope, _) = scheduler.scopes.open();
        let mut worker = Worker::new(Arc::new(failing_release(false)))
            .expect("worker")
            .with_scheduler(Some(scheduler.clone()));
        worker.task_id = 1;
        worker.owner_scope = scheduler.scopes.enroll(scope, 1);
        scheduler.worker_started(1);
        let _ticket = scheduler.worker_ticket(1);
        for _ in 0..5 {
            assert!(matches!(
                worker.dispatch_one().unwrap(),
                DispatchStep::Continue
            ));
        }
        if vm_shutdown {
            scheduler.request_cancel();
        } else {
            scheduler.scopes.cancel(scope);
        }
        let error = loop {
            match worker.dispatch_one() {
                Err(error) => break error,
                Ok(DispatchStep::Continue) => {}
                Ok(_) => panic!("release failure must end the task"),
            }
        };
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
        assert!(error.message.contains("panic: 2"), "{error:?}");
        assert!(
            worker
                .hosted
                .supplemental_diagnostics
                .lock()
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn propagated_error_and_none_survive_release_failure_with_a_diagnostic() {
    for none in [false, true] {
        let outcome = if none {
            abc(Opcode::MakeNone, 2, 0, 0, 0)
        } else {
            abc(Opcode::MakeError, 2, 0, 0, 0)
        };
        let executable = image(
            vec![
                abc(Opcode::Return, crate::NO_REGISTER, 0, 0, 0),
                abc(Opcode::EnterCleanupScope, 0, 0, 0, 0),
                abc(Opcode::BeginResourceInit, 0, 0, 0, 0),
                abx(Opcode::LoadConstant, 0, 0),
                abc(Opcode::MakeClosure, 1, 2, 0, 0),
                abc(Opcode::RegisterCleanup, 0, 1, 0, 0),
                outcome,
                abc(Opcode::ExitCleanupScope, 2, 0, 0, 1),
                abc(Opcode::Return, 2, 0, 0, 0),
                abx(Opcode::LoadConstant, 1, 1),
                abc(Opcode::Panic, 1, 0, 0, 0),
                abc(Opcode::Return, crate::NO_REGISTER, 0, 0, 0),
            ],
            vec![Constant::Integer(1), Constant::Integer(2)],
            &[
                FunctionSpec {
                    start: 0,
                    end: 1,
                    arity: 0,
                    captures: 0,
                    registers: 0,
                    returns: ReturnConvention::Unit,
                },
                FunctionSpec {
                    start: 1,
                    end: 9,
                    arity: 0,
                    captures: 0,
                    registers: 3,
                    returns: ReturnConvention::Value,
                },
                FunctionSpec {
                    start: 9,
                    end: 12,
                    arity: 1,
                    captures: 0,
                    registers: 2,
                    returns: ReturnConvention::Unit,
                },
            ],
        );
        let mut vm = Vm::new(executable);
        let value = vm
            .call(crate::FunctionId::new(1), Vec::new())
            .expect("propagated value remains intact")
            .value
            .try_to_data()
            .unwrap();
        assert!(matches!(
            (&value, none),
            (crate::Value::OptionNone, true) | (crate::Value::ResultError(_), false)
        ));
        let diagnostics = vm.take_supplemental_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("panic: 2"));
    }
}

#[test]
fn cancelled_scope_child_finishes_suspending_release_on_one_worker() {
    let source = r#"program T;
uses Std.Console as Console, Std.Time as Time, Std.Tasks as Tasks;
type Resource = record
  release procedure Close(Self: Resource);
  begin Time.Sleep(5); Console.WriteLn('released'); end procedure;
end record;
procedure Child(Ready: channel of integer);
begin
  using const Item := Resource();
  discard Tasks.Send(Ready, 1);
  Time.Sleep(100000);
end procedure;
begin scope
  const Ready: channel of integer := Tasks.CreateChannel(1);
  discard go Child(Ready);
  discard Tasks.Receive(Ready).Unwrap();
  panic('parent');
end scope; end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.pool_size = 1;
    let error = vm.run().expect_err("parent panics");
    assert!(error.message.contains("parent"), "{error:?}");
    assert_eq!(vm.output().lines, ["released"]);
}

#[test]
fn body_panic_releases_all_resources_in_reverse_order_after_release_panic() {
    let source = r#"program T;
uses Std.Console as Console;
type Resource = record
  Number: integer;
  release procedure Close(Self: Resource);
  begin
    Console.WriteLn(Self.Number);
    if Self.Number = 2 then panic('release'); end if;
  end procedure;
end record;
begin
  using const A := Resource(Number := 1);
  using const B := Resource(Number := 2);
  panic('body');
end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = Vm::new(fpas_compiler::compile(&program).expect("compile"));
    let error = vm.run().expect_err("body panics");
    assert!(error.message.contains("body"), "{error:?}");
    assert_eq!(vm.output().lines, ["2", "1"]);
    let diagnostics = vm.take_supplemental_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("release"));
}

#[test]
fn protected_result_does_not_resume_a_disposer_after_nested_release_failure() {
    let source = r#"program T;
uses Std.Console as Console;
type Inner = record
  release procedure Close(Self: Inner); begin panic('inner release'); end procedure;
end record;
type Outer = record
  release procedure Close(Self: Outer);
  begin
    begin using const Item := Inner(); end;
    Console.WriteLn('must not resume');
  end procedure;
end record;
function Work(): result of (integer, string);
begin using const Item := Outer(); return Error('original'); end function;
begin if not (Work() is Error(_)) then panic('result changed'); end if; end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.run().expect("original Result.Error is preserved");
    assert!(vm.output().lines.is_empty());
    let diagnostics = vm.take_supplemental_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("inner release"));

    let successful_return = source.replace("return Error('original');", "return Ok(1);");
    let (program, errors) = fpas_parser::parse(&successful_return);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = Vm::new(fpas_compiler::compile(&program).expect("compile"));
    let error = vm.run().expect_err("release failure overrides success");
    assert!(error.message.contains("inner release"), "{error:?}");
    assert!(vm.output().lines.is_empty());
    assert!(vm.take_supplemental_diagnostics().is_empty());
}

#[test]
fn cancelling_nested_scope_joins_children_before_outer_resource_release() {
    let source = r#"program T;
uses Std.Console as Console, Std.Time as Time, Std.Tasks as Tasks;
type Resource = record
  Number: integer;
  release procedure Close(Self: Resource);
  begin Console.WriteLn(Self.Number); end procedure;
end record;
procedure Child(Ready: channel of integer);
begin
  using const Item := Resource(Number := 2);
  discard Tasks.Send(Ready, 1);
  Time.Sleep(100000);
end procedure;
begin
  using const Outer := Resource(Number := 1);
  scope
    const Ready: channel of integer := Tasks.CreateChannel(1);
    discard go Child(Ready);
    discard Tasks.Receive(Ready).Unwrap();
    panic('body');
  end scope;
end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.pool_size = 1;
    let error = vm.run().expect_err("body panics");
    assert!(error.message.contains("body"), "{error:?}");
    assert_eq!(vm.output().lines, ["2", "1"]);
}

#[test]
fn shutdown_drains_suspended_child_before_parent_resource() {
    let source = r#"program T;
uses Std.Console as Console, Std.Time as Time, Std.Tasks as Tasks;
type Resource = record
  Number: integer;
  release procedure Close(Self: Resource);
  begin Time.Sleep(5); Console.WriteLn(Self.Number); end procedure;
end record;
procedure Child(Ready: channel of integer);
begin
  using const Item := Resource(Number := 2);
  discard Tasks.Send(Ready, 1);
  Time.Sleep(100000);
end procedure;
begin
  using const Outer := Resource(Number := 1);
  scope
    const Ready: channel of integer := Tasks.CreateChannel(1);
    discard go Child(Ready);
    discard Tasks.Receive(Ready).Unwrap();
    Console.WriteLn('parent-ready');
    Time.Sleep(100000);
  end scope;
end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).expect("compile");
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let (ready_tx, ready_rx) = mpsc::channel();
    let writer = Capture {
        bytes: Arc::clone(&bytes),
        ready: ready_tx,
    };
    let mut vm = Vm::with_writer(image, Box::new(writer));
    vm.pool_size = 1;
    let shutdown = vm.shutdown_handle();
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(vm.run());
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("both resources acquired");
    shutdown.shutdown();
    let error = done_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("shutdown completes")
        .expect_err("cancelled");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN);
    let output = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    assert!(output.contains("2\n1\n"), "{output:?}");
}

#[test]
fn shutdown_closes_native_file_before_outer_release() {
    let path = std::env::temp_dir().join(format!(
        "fpas-resource-shutdown-{}-{}.txt",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    std::fs::write(&path, b"x").unwrap();
    let path_source = path
        .to_string_lossy()
        .replace('\\', "/")
        .replace('\'', "''");
    let source = format!(
        r#"program T;
uses Std.Console as Console, Std.Time as Time, Std.Io.Fs as Fs;
type Watcher = record
  File: Fs.File;
  release procedure Close(Self: Watcher);
  begin
    if Fs.Tell(Self.File) is Error(Fs.FsError.Closed(_)) then
      Console.WriteLn('file-closed');
    else
      panic('file remained open');
    end if;
  end procedure;
end record;
begin
  const File := Fs.Open('{path_source}', Fs.FileMode.Read).Unwrap();
  using const Outer := Watcher(File := File);
  begin
    using const Inner := File;
    Console.WriteLn('parent-ready');
    scope Time.Sleep(100000); end scope;
  end;
end."#
    );
    let (program, errors) = fpas_parser::parse(&source);
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).expect("compile");
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let (ready_tx, ready_rx) = mpsc::channel();
    let mut vm = Vm::with_writer(
        image,
        Box::new(Capture {
            bytes: Arc::clone(&bytes),
            ready: ready_tx,
        }),
    );
    vm.pool_size = 1;
    let shutdown = vm.shutdown_handle();
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(vm.run());
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("native file acquired");
    shutdown.shutdown();
    let result = done_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("shutdown completes");
    std::fs::remove_file(&path).unwrap();
    assert!(result.is_err());
    let output = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    assert!(output.contains("file-closed"), "{output:?}");
}

#[test]
fn cancellation_after_producer_commit_waits_for_registration_only_on_success() {
    use crate::execution::vm::{dispatch::DispatchStep, tasks::TaskScheduler, worker::Worker};

    for (failed, immediate) in [(false, false), (true, false), (false, true)] {
        let result = if failed {
            Opcode::MakeError
        } else {
            Opcode::Move
        };
        let producer = if immediate {
            abx(Opcode::LoadConstant, 0, 0)
        } else {
            abc(Opcode::CallDirect, 0, 1, 0, 0)
        };
        let executable = image(
            vec![
                abc(Opcode::EnterCleanupScope, 0, 0, 0, 0),
                abc(Opcode::BeginResourceInit, 0, 0, 0, 0),
                abc(Opcode::ArmResourceInit, 0, 0, 0, 0),
                producer,
                abc(Opcode::MakeClosure, 1, 2, 0, 0),
                abc(Opcode::RegisterCleanup, 0, 1, 0, 0),
                abc(Opcode::ExitCleanupScope, crate::NO_REGISTER, 0, 0, 0),
                abc(Opcode::Return, crate::NO_REGISTER, 0, 0, 0),
                abx(Opcode::LoadConstant, 0, 0),
                abc(result, 1, 0, 0, 0),
                abc(Opcode::Return, 1, 0, 0, 0),
                abx(Opcode::LoadConstant, 1, 1),
                abc(Opcode::Panic, 1, 0, 0, 0),
                abc(Opcode::Return, crate::NO_REGISTER, 0, 0, 0),
            ],
            vec![Constant::Integer(1), Constant::Integer(2)],
            &[
                FunctionSpec {
                    start: 0,
                    end: 8,
                    arity: 0,
                    captures: 0,
                    registers: 2,
                    returns: ReturnConvention::Unit,
                },
                FunctionSpec {
                    start: 8,
                    end: 11,
                    arity: 0,
                    captures: 0,
                    registers: 2,
                    returns: ReturnConvention::Value,
                },
                FunctionSpec {
                    start: 11,
                    end: 14,
                    arity: 1,
                    captures: 0,
                    registers: 2,
                    returns: ReturnConvention::Unit,
                },
            ],
        );
        let scheduler = Arc::new(TaskScheduler::new());
        let (scope, _) = scheduler.scopes.open();
        let mut worker = Worker::new(Arc::new(executable))
            .unwrap()
            .with_scheduler(Some(scheduler.clone()));
        worker.task_id = 1;
        worker.owner_scope = scheduler.scopes.enroll(scope, 1);
        for _ in 0..if immediate { 4 } else { 7 } {
            assert!(matches!(
                worker.dispatch_one().unwrap(),
                DispatchStep::Continue
            ));
        }
        scheduler.scopes.cancel(scope);
        let error = loop {
            match worker.dispatch_one() {
                Err(error) => break error,
                Ok(DispatchStep::Continue) => {}
                Ok(_) => panic!("cancellation must fail the task"),
            }
        };
        if failed {
            assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED);
        } else {
            assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
            assert!(error.message.contains("panic: 2"));
        }
        let diagnostics = worker.hosted.supplemental_diagnostics.lock().unwrap();
        assert!(diagnostics.is_empty(), "failed={failed}");
    }
}
