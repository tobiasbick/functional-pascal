//! Native socket jobs join and release saved frames, cycles and unused cases.

use crate::measurement::completed_threads;
use crate::workloads::{image, schedule};
use fpas_runtime::{CollectionSchedule, VerifiedExecutable, execution::Vm, profile};
use std::io::{self, Write};
use std::sync::{Arc, OnceLock, Weak, mpsc};
use std::time::Duration;

const SOURCE: &str = include_str!("../../src/execution/vm/hosted/net/fixtures/pool_control.fpas");

struct PendingWriter {
    stop: Arc<OnceLock<Weak<fpas_runtime::execution::ShutdownHandle>>>,
    shutdown: bool,
}

impl Write for PendingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        assert!(bytes == b"pending" || bytes == b"locked");
        // Keep this task at its native output boundary until a peer has parked.
        // Configuration handshakes prevent its trigger from overtaking the
        // callback's initial output and blocking that callback on the console.
        crate::shutdown::wait_for_park();
        if self.shutdown && bytes == b"pending" {
            self.stop.get().unwrap().upgrade().unwrap().shutdown();
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn socket_wait_teardown() {
    let executable = image(SOURCE);
    for config in [CollectionSchedule::default(), schedule()] {
        for mode in [
            "read_cancel",
            "accept_cancel",
            "accept_close",
            "timeout",
            "scope_exit",
            "vm_shutdown",
        ] {
            completed_threads(&format!("native socket teardown {mode}"), 4, || {
                exercise(&executable, config, vec![mode.into()]);
            });
        }
    }
}

pub fn socket_connect_completion() {
    let executable = image(include_str!(
        "../../src/execution/vm/hosted/net/fixtures/pool_progress.fpas"
    ));
    for config in [CollectionSchedule::default(), schedule()] {
        for cancellable in [false, true] {
            completed_threads("native connect and DNS completion", 4, || {
                exercise(
                    &executable,
                    config,
                    if cancellable {
                        vec!["cancellable".into()]
                    } else {
                        Vec::new()
                    },
                );
            });
        }
    }
}

pub fn socket_configuration_teardown() {
    let executable = image(include_str!(
        "../../src/execution/vm/hosted/net/fixtures/pool_config.fpas"
    ));
    for config in [CollectionSchedule::default(), schedule()] {
        for mode in ["normal", "scope_exit", "vm_shutdown"] {
            completed_threads(&format!("native socket configuration {mode}"), 4, || {
                exercise(&executable, config, vec![mode.into()]);
            });
        }
    }
}

fn exercise(executable: &VerifiedExecutable, config: CollectionSchedule, arguments: Vec<String>) {
    let shutdown = arguments.first().is_some_and(|mode| mode == "vm_shutdown");
    let counters = profile::begin();
    let stop_slot = Arc::new(OnceLock::new());
    let mut vm = Vm::with_writer_and_args(
        executable.clone(),
        Box::new(PendingWriter {
            stop: stop_slot.clone(),
            shutdown,
        }),
        arguments,
    )
    .with_collection_schedule(config);
    let stop = Arc::new(vm.shutdown_handle());
    assert!(stop_slot.set(Arc::downgrade(&stop)).is_ok());
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
    let result = outcome.expect("native socket teardown must finish");
    if shutdown {
        let error = result.expect_err("VM shutdown");
        assert!(
            matches!(
                error.code,
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                    | fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
            ),
            "{error:?}"
        );
    } else {
        result.expect("network result");
    }
    running.join().unwrap();
    drop(stop);
    drop(counters.finish());
}
