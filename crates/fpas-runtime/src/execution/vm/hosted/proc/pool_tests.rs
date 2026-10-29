//! Real child gates require another FPAS task to run on the same pool worker.

#[path = "fixtures/child.rs"]
mod child;
#[path = "fixtures/observer.rs"]
mod observer;

use crate::CollectionSchedule;
use crate::execution::vm::{ShutdownHandle, Vm};
use std::io::{self, Write};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, OnceLock, Weak, mpsc};
use std::time::Duration;

const SOURCE: &str = include_str!("fixtures/pool.fpas");
const CHILD: &str = "execution::vm::hosted::proc::pool_tests::child_process";

struct PendingWriter {
    stop: Arc<OnceLock<Weak<ShutdownHandle>>>,
    shutdown: bool,
}

impl Write for PendingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        assert_eq!(bytes, b"pending");
        if self.shutdown {
            self.stop.get().unwrap().upgrade().unwrap().shutdown();
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn run(mode: &str, config: CollectionSchedule) {
    run_source(SOURCE, mode, config);
}

fn run_source(source: &str, mode: &str, config: CollectionSchedule) {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    let observer = observer::Observer::new();
    let stop_slot = Arc::new(OnceLock::new());
    let shutdown = matches!(
        mode,
        "vm_shutdown" | "receiver_shutdown" | "select_vm_shutdown"
    );
    let mut vm = Vm::with_writer_and_args(
        image,
        Box::new(PendingWriter {
            stop: stop_slot.clone(),
            shutdown: matches!(mode, "vm_shutdown" | "select_vm_shutdown"),
        }),
        vec![
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            CHILD.into(),
            mode.into(),
            observer.port(),
        ],
    )
    .with_collection_schedule(config);
    vm.pool_size = 1;
    let hosted = Arc::downgrade(&vm.hosted);
    let stop = Arc::new(vm.shutdown_handle());
    assert!(stop_slot.set(Arc::downgrade(&stop)).is_ok());
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = lines.clone();
    let receiver_stop = stop_slot.clone();
    let receiver_shutdown = mode == "receiver_shutdown";
    let receiver_returned = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let returned = receiver_returned.clone();
    vm.set_program_stderr(Arc::new(move |line| {
        sink.lock().unwrap().push(line.to_owned());
        if receiver_shutdown {
            receiver_stop.get().unwrap().upgrade().unwrap().shutdown();
            returned.store(true, std::sync::atomic::Ordering::Release);
        }
    }));
    let (send, receive) = mpsc::channel();
    let running = std::thread::spawn(move || {
        let outcome = vm.run();
        drop(vm);
        let _ = send.send(outcome);
    });
    let outcome = receive.recv_timeout(Duration::from_secs(30));
    if outcome.is_err() {
        stop.shutdown();
    }
    let result = outcome.expect("process call must release the only pool worker");
    running.join().unwrap();
    if shutdown {
        let error = result.expect_err("explicit VM stop");
        assert!(
            matches!(
                error.code,
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                    | fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
            ),
            "{error:?}"
        );
    } else {
        result.expect("process fixture");
    }
    if matches!(mode, "run" | "options" | "receiver_shutdown") {
        assert_eq!(*lines.lock().unwrap(), ["prefix"]);
    }
    if receiver_shutdown {
        assert!(
            receiver_returned.load(std::sync::atomic::Ordering::Acquire),
            "native stderr shutdown callback must return without a self-join panic"
        );
    }
    observer.assert_closed(if child::descendants(mode) { 2 } else { 1 });
    drop(stop);
    assert!(
        hosted.upgrade().is_none(),
        "saved process states retained their VM"
    );
}

#[test]
fn one_worker_runs_plain_capture_and_options_calls_inside_nested_callbacks() {
    for config in [
        CollectionSchedule::default(),
        CollectionSchedule {
            allocations: NonZeroUsize::new(1).unwrap(),
            mutations: NonZeroUsize::new(1).unwrap(),
            safe_points: NonZeroUsize::new(1).unwrap(),
        },
    ] {
        for mode in [
            "run",
            "capture",
            "options",
            "capture_options",
            "descendants",
        ] {
            run(mode, config);
        }
    }
}

#[test]
fn pending_process_calls_cancel_timeout_enforce_limits_and_reap_on_scope_or_vm_exit() {
    for mode in [
        "cancel",
        "timeout",
        "limit",
        "scope_exit",
        "vm_shutdown",
        "receiver_shutdown",
    ] {
        run(mode, CollectionSchedule::default());
    }
}

#[test]
fn all_stop_debugger_runs_process_peers_and_reaps_abandoned_children() {
    use crate::execution::vm::debug::{DebugRunResult, DebugSession};
    for mode in ["capture_options", "descendants", "cancel", "scope_exit"] {
        let observer = observer::Observer::new();
        let (program, errors) = fpas_parser::parse(SOURCE);
        assert!(errors.is_empty(), "{errors:?}");
        let image = fpas_compiler::compile(&program).unwrap();
        let mut session = DebugSession::with_args(
            image,
            vec![
                std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                CHILD.into(),
                mode.into(),
                observer.port(),
            ],
        )
        .unwrap();
        assert!(
            matches!(
                session.continue_execution().unwrap(),
                DebugRunResult::Terminated(_)
            ),
            "process debugger fixture stopped: {mode}"
        );
        observer.assert_closed(if child::descendants(mode) { 2 } else { 1 });
    }
}

#[test]
fn selectable_pipes_release_one_worker_preserve_binary_io_and_abandon_cases() {
    for config in [
        CollectionSchedule::default(),
        CollectionSchedule {
            allocations: NonZeroUsize::new(1).unwrap(),
            mutations: NonZeroUsize::new(1).unwrap(),
            safe_points: NonZeroUsize::new(1).unwrap(),
        },
    ] {
        for mode in [
            "select",
            "select_cancel",
            "select_timeout",
            "select_scope_exit",
            "select_vm_shutdown",
        ] {
            run_source(include_str!("fixtures/selection.fpas"), mode, config);
        }
    }
}

#[test]
fn all_stop_debugger_selects_interactive_pipes() {
    use crate::execution::vm::debug::{DebugRunResult, DebugSession};
    let source = include_str!("fixtures/selection.fpas");
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    for mode in [
        "select",
        "select_cancel",
        "select_timeout",
        "select_scope_exit",
    ] {
        let observer = observer::Observer::new();
        let mut session = DebugSession::with_args(
            image.clone(),
            vec![
                std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                CHILD.into(),
                mode.into(),
                observer.port(),
            ],
        )
        .unwrap();
        assert!(matches!(
            session.continue_execution().unwrap(),
            DebugRunResult::Terminated(_)
        ));
        drop(session);
        observer.assert_closed(1);
    }
}

#[test]
#[ignore = "owned subprocess fixture, launched by pool process progress and cleanup tests"]
fn child_process() {
    child::run();
}
