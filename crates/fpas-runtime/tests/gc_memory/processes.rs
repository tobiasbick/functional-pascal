//! Exact balances for live process waits, callback cycles and owned descendants.

#[path = "../../src/execution/vm/hosted/proc/fixtures/child.rs"]
pub(super) mod child;
#[path = "../../src/execution/vm/hosted/proc/fixtures/observer.rs"]
mod observer;

use crate::measurement::completed_threads;
use crate::workloads::{image, schedule};
use fpas_runtime::{
    CollectionSchedule,
    execution::{ShutdownHandle, Vm},
};
use std::io::{self, Write};
use std::sync::{Arc, OnceLock, Weak, mpsc};
use std::time::Duration;

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

pub fn owned_process_teardown() {
    teardown(
        include_str!("../../src/execution/vm/hosted/proc/fixtures/pool.fpas"),
        &[
            "run",
            "capture",
            "options",
            "capture_options",
            "descendants",
            "cancel",
            "scope_exit",
            "vm_shutdown",
            "receiver_shutdown",
        ],
    );
}

pub fn selectable_pipe_teardown() {
    teardown(
        include_str!("../../src/execution/vm/hosted/proc/fixtures/selection.fpas"),
        &[
            "select",
            "select_cancel",
            "select_timeout",
            "select_scope_exit",
            "select_vm_shutdown",
        ],
    );
}

fn teardown(source: &str, modes: &[&'static str]) {
    let executable = image(source);
    for config in [CollectionSchedule::default(), schedule()] {
        for &mode in modes {
            completed_threads(&format!("owned process teardown {mode}"), 4, || {
                let observer = observer::Observer::new();
                let stop_slot = Arc::new(OnceLock::new());
                let mut vm = Vm::with_writer_and_args(
                    executable.clone(),
                    Box::new(PendingWriter {
                        stop: stop_slot.clone(),
                        shutdown: matches!(mode, "vm_shutdown" | "select_vm_shutdown"),
                    }),
                    vec![
                        std::env::current_exe()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned(),
                        "process_child".into(),
                        mode.into(),
                        observer.port(),
                    ],
                )
                .with_collection_schedule(config);
                let stop = Arc::new(vm.shutdown_handle());
                assert!(stop_slot.set(Arc::downgrade(&stop)).is_ok());
                let callback_stop = stop_slot.clone();
                vm.set_program_stderr(Arc::new(move |_| {
                    if mode == "receiver_shutdown" {
                        callback_stop.get().unwrap().upgrade().unwrap().shutdown();
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
                let outcome = outcome.expect("owned process teardown must finish");
                if matches!(
                    mode,
                    "vm_shutdown" | "receiver_shutdown" | "select_vm_shutdown"
                ) {
                    let error = outcome.expect_err("VM stop");
                    assert!(
                        matches!(
                            error.code,
                            fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                                | fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
                        ),
                        "{error:?}"
                    );
                } else {
                    outcome.expect("process result");
                }
                running.join().unwrap();
                observer.assert_closed(if child::descendants(mode) { 2 } else { 1 });
                drop(stop);
            });
        }
    }
}
