//! File ownership ends at VM/session boundaries even when shared host state survives.

#![allow(
    clippy::expect_used,
    reason = "lifecycle fixtures require concrete host outcomes"
)]

use super::*;
use crate::vm::Vm;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const HELPERS: &str = r#"
function OpenFile(Path: string): Fs.File;
begin
  return Fs.Open(Path, Fs.FileMode.Read).Unwrap();
end function;
function Position(File: Fs.File): integer;
begin
  return Fs.Tell(File).Unwrap();
end function;
function ReadOne(File: Fs.File): integer;
begin
  const Bytes := Fs.Read(File, 1).Unwrap();
  return Bytes[0];
end function;
function IsClosed(File: Fs.File): boolean;
begin
  return (Fs.Tell(File) is Error(Fs.FsError.Closed('Std.Fs.Tell'))) and
    (Fs.Read(File, 1) is Error(Fs.FsError.Closed('Std.Fs.Read')));
end function;
"#;

struct InputFile(PathBuf);

impl InputFile {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "fpas-file-lifecycle-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, [42, 43]).expect("create input file");
        Self(path)
    }

    fn argument(&self) -> Value {
        Value::Str(self.0.to_string_lossy().into_owned().into())
    }
}

impl Drop for InputFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn executable(main: &str) -> VerifiedExecutable {
    let source = format!(
        "program FileLifecycle; uses Std.Fs as Fs; var Saved: option of Fs.File := None; {HELPERS} begin {main} end."
    );
    let (program, errors) = fpas_parser::parse(&source);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program).expect("compile real file lifecycle source")
}

fn function(executable: &VerifiedExecutable, name: &str) -> FunctionId {
    let image = executable.executable();
    let index = image
        .functions
        .iter()
        .position(|function| {
            image.strings.get(function.name).is_some_and(|declared| {
                declared
                    .rsplit('.')
                    .next()
                    .is_some_and(|leaf| leaf.eq_ignore_ascii_case(name))
            })
        })
        .expect("source callback function");
    FunctionId::new(u16::try_from(index).expect("function index"))
}

fn probe(
    executable: Arc<VerifiedExecutable>,
    layouts: Arc<RuntimeLayouts>,
    globals: Arc<RwLock<Vec<Option<Value>>>>,
    hosted: Arc<HostedState>,
    file: Value,
) {
    let outcome = Worker::for_function_with_state(
        Arc::clone(&executable),
        function(&executable, "IsClosed"),
        vec![file],
        globals,
        layouts,
        hosted,
    )
    .expect("shared-state closure probe")
    .run()
    .expect("closed handles remain operational outcomes");
    assert_eq!(outcome.value, Value::Boolean(true));
}

#[test]
fn callbacks_keep_files_between_invocations_and_close_on_cancel_shutdown_or_drop() {
    for action in ["cancel", "shutdown", "drop"] {
        let input = InputFile::new();
        let mut session = CallbackSession::new(executable(""));
        let image = Arc::clone(&session.executable);
        let file = session
            .invoke(function(&image, "OpenFile"), vec![input.argument()])
            .expect("open through real callback")
            .value;
        assert_eq!(
            session
                .invoke(function(&image, "Position"), vec![file.clone()])
                .expect("position survives previous invocation")
                .value,
            Value::Integer(0)
        );
        assert_eq!(
            session
                .invoke(function(&image, "ReadOne"), vec![file.clone()])
                .expect("read through another callback")
                .value,
            Value::Integer(42)
        );
        assert_eq!(
            session
                .invoke(function(&image, "Position"), vec![file.clone()])
                .expect("shared file cursor")
                .value,
            Value::Integer(1)
        );
        // Keep the registry and executable alive so Arc destruction cannot satisfy closure.
        let hosted = Arc::clone(&session.hosted);
        let globals = Arc::clone(&session.globals);
        let layouts = session.layouts.clone().expect("admitted layouts");
        if action == "drop" {
            drop(session);
        } else {
            if action == "cancel" {
                session.cancel();
            } else {
                session.shutdown();
            }
            probe(
                Arc::clone(&image),
                Arc::clone(&layouts),
                Arc::clone(&globals),
                Arc::clone(&hosted),
                file.clone(),
            );
            assert_eq!(
                session
                    .invoke(function(&image, "Position"), vec![file.clone()])
                    .expect_err("stopped session")
                    .code,
                RUNTIME_VM_SHUTDOWN
            );
            drop(session);
        }
        probe(image, layouts, globals, hosted, file);
    }
}

#[test]
fn zero_worker_vm_closes_files_on_success_and_failure_before_vm_or_shutdown_handle_drop() {
    for fail in [false, true] {
        let input = InputFile::new();
        let path = input.0.to_string_lossy().replace('\'', "''");
        let main = format!(
            "Saved := Some(OpenFile('{path}')); {}",
            if fail {
                "panic('lifecycle failure');"
            } else {
                ""
            }
        );
        let mut vm = Vm::new(executable(&main));
        vm.pool_size = 0;
        // Keep source globals in shared storage so a real source-opened handle can be inspected.
        vm.use_local_globals = false;
        let shutdown = vm.shutdown_handle();
        let outcome = vm.run();
        if fail {
            assert_eq!(
                outcome.expect_err("program panic").code,
                fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC
            );
        } else {
            outcome.expect("successful root");
        }
        let index = vm
            .executable
            .executable()
            .globals
            .iter()
            .position(|global| {
                vm.executable
                    .executable()
                    .strings
                    .get(global.name)
                    .is_some_and(|name| {
                        name.rsplit('.')
                            .next()
                            .is_some_and(|leaf| leaf.eq_ignore_ascii_case("Saved"))
                    })
            })
            .expect("retained source global");
        let file = match vm.globals.read().expect("source global storage")[index].as_ref() {
            Some(Value::OptionSome(file)) => file.as_ref().clone(),
            other => panic!("source did not open file: {other:?}"),
        };
        probe(
            Arc::clone(&vm.executable),
            vm.layouts.clone().expect("admitted layouts"),
            Arc::clone(&vm.globals),
            Arc::clone(&vm.hosted),
            file,
        );
        drop(shutdown);
        drop(vm);
    }
}

#[test]
fn one_shot_vm_call_closes_returned_file_before_vm_or_shutdown_handle_drop() {
    let input = InputFile::new();
    let mut vm = Vm::new(executable(""));
    let target = function(&vm.executable, "OpenFile");
    let shutdown = vm.shutdown_handle();
    let file = vm
        .call(target, vec![input.argument()])
        .expect("one-shot callback opens a real file")
        .value;
    probe(
        Arc::clone(&vm.executable),
        vm.layouts.clone().expect("admitted layouts"),
        Arc::clone(&vm.globals),
        Arc::clone(&vm.hosted),
        file,
    );
    drop(shutdown);
    drop(vm);
}
