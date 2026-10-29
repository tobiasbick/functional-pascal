//! Exact allocation balances after stopping a VM with real parked pool tasks.

use crate::measurement::completed_threads;
use crate::workloads::{image, schedule};
use fpas_runtime::execution::{ShutdownHandle, Vm};
use fpas_runtime::profile::{self, Counter};
use fpas_runtime::{CollectionSchedule, VerifiedExecutable};
use std::io::{self, Write};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const SOURCE: &str = include_str!("fixtures/shutdown.fpas");
const DEADLINE: Duration = Duration::from_secs(30);
const MODES: [&str; 11] = [
    "receive",
    "send",
    "receive_cancel",
    "send_cancel",
    "sleep",
    "select",
    "wait",
    "wait_any",
    "wait_all",
    "scope_join",
    "group_join",
];

struct BarrierWriter {
    entered: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
    first: bool,
}

impl Write for BarrierWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        assert!(bytes == b"root" || bytes == b"leaf", "unexpected output");
        if self.first {
            self.first = false;
            self.entered.send(()).map_err(io::Error::other)?;
            self.release.recv().map_err(io::Error::other)?;
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Always unblock the host call and request shutdown, including on assertion failure.
struct Stop {
    shutdown: ShutdownHandle,
    release: mpsc::Sender<()>,
}

impl Drop for Stop {
    fn drop(&mut self) {
        self.shutdown.shutdown();
        let _ = self.release.send(());
    }
}

pub(super) fn wait_for_park() {
    let deadline = Instant::now() + DEADLINE;
    loop {
        // The surrounding profiling run keeps counting enabled. These overlapping
        // snapshots inspect existing counters without resetting them or adding hooks.
        let snapshot = profile::begin().finish();
        let parks = snapshot
            .counts
            .iter()
            .find(|(counter, _)| *counter == Counter::TaskParks)
            .unwrap()
            .1;
        if parks != 0 {
            return;
        }
        assert!(Instant::now() < deadline, "worker never parked");
        std::thread::yield_now();
    }
}

fn run(executable: &VerifiedExecutable, config: CollectionSchedule, mode: &str, action: &str) {
    let profile = profile::begin();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let mut vm = Vm::with_writer_and_args(
        executable.clone(),
        Box::new(BarrierWriter {
            entered: entered_tx,
            release: release_rx,
            first: true,
        }),
        vec![mode.into(), action.into()],
    )
    .with_collection_schedule(config);
    assert_ne!(
        vm.worker_count(),
        0,
        "fixture must use the actual worker pool"
    );
    let stop = Stop {
        shutdown: vm.shutdown_handle(),
        release: release_tx,
    };
    let running = std::thread::spawn(move || {
        let result = vm.run();
        drop(vm);
        let _ = finished_tx.send(result);
    });
    entered_rx.recv_timeout(DEADLINE).unwrap();
    // The root and any Leaf are blocked in the host writer/console mutex.
    // Only Block can produce this park, and none of its wait sources is ready.
    wait_for_park();
    assert!(!running.is_finished(), "pending operation completed early");
    if action == "shutdown" {
        stop.shutdown.shutdown();
        stop.shutdown.shutdown();
    }
    stop.release.send(()).unwrap();
    // The test thread does not own a scoped join: a broken shutdown must fail
    // this isolated child within a bound instead of hanging the whole suite.
    let outcome = finished_rx.recv_timeout(DEADLINE).unwrap();
    running.join().unwrap();
    match action {
        "shutdown" => {
            let error = outcome.unwrap_err();
            // Scope cancellation can win the next instruction before the outer
            // dispatch batch observes the VM-wide stop request.
            assert!(
                matches!(
                    error.code,
                    fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                        | fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
                ),
                "{error:?}"
            );
        }
        "panic" => {
            let error = outcome.unwrap_err();
            assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
            assert!(
                error.message.contains("expected pending-task failure"),
                "{error:?}"
            );
            assert!(
                !error
                    .help
                    .as_deref()
                    .unwrap_or_default()
                    .contains("callback ran"),
                "unexpected secondary callback failure: {error:?}"
            );
        }
        "break" => drop(outcome.unwrap()),
        _ => unreachable!(),
    }
    drop(stop);
    drop(profile.finish());
}

fn matrix(action: &str) {
    let executable = image(SOURCE);
    for config in [CollectionSchedule::default(), schedule()] {
        for mode in MODES {
            completed_threads(&format!("pending {mode}, {action}"), 4, || {
                run(&executable, config, mode, action);
            });
        }
    }
}

pub fn explicit_shutdown() {
    matrix("shutdown");
}

pub fn runtime_failure() {
    matrix("panic");
}

pub fn scope_exit() {
    matrix("break");
}
