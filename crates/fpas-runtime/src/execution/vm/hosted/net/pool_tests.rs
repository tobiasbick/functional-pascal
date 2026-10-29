//! Socket waits and nested callbacks make progress with exactly one VM pool worker.

use crate::CollectionSchedule;
use crate::execution::vm::hosted::HostedState;
use crate::execution::vm::{ShutdownHandle, Vm};
use std::io::{self, Write};
use std::num::NonZeroUsize;
use std::sync::{Arc, OnceLock, Weak};
use std::sync::{atomic::Ordering, mpsc};
use std::time::Duration;

struct PendingWriter {
    state: Arc<OnceLock<Weak<HostedState>>>,
    stop: Arc<OnceLock<Weak<ShutdownHandle>>>,
    shutdown: bool,
}

impl Write for PendingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        assert_eq!(bytes, b"pending");
        let hosted = self
            .state
            .get()
            .expect("host initialized")
            .upgrade()
            .expect("live VM");
        assert!(
            hosted.network_jobs.load(Ordering::Acquire) > 0,
            "trigger must run while another task owns native socket work"
        );
        if self.shutdown {
            self.stop
                .get()
                .expect("host stop initialized")
                .upgrade()
                .expect("live stop handle")
                .shutdown();
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn run(source: &str, arguments: Vec<String>, config: CollectionSchedule) {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).expect("fixture");
    let state = Arc::new(OnceLock::new());
    let stop_slot = Arc::new(OnceLock::new());
    let shutdown_requested = arguments.first().is_some_and(|mode| mode == "vm_shutdown");
    let mut vm = Vm::with_writer_and_args(
        image,
        Box::new(PendingWriter {
            state: state.clone(),
            stop: stop_slot.clone(),
            shutdown: shutdown_requested,
        }),
        arguments,
    )
    .with_collection_schedule(config);
    assert!(state.set(Arc::downgrade(&vm.hosted)).is_ok());
    vm.pool_size = 1;
    let shutdown = Arc::new(vm.shutdown_handle());
    assert!(stop_slot.set(Arc::downgrade(&shutdown)).is_ok());
    let (sender, receiver) = mpsc::channel();
    let running = std::thread::spawn(move || {
        let outcome = vm.run();
        let jobs = vm.hosted.network_jobs.load(Ordering::Acquire);
        let _ = sender.send((outcome, jobs));
    });
    let result = receiver.recv_timeout(Duration::from_secs(30));
    if result.is_err() {
        shutdown.shutdown();
    }
    let (outcome, jobs) = result.expect("socket waits must release the only pool worker");
    running.join().expect("VM thread");
    if shutdown_requested {
        let error = outcome.expect_err("explicit VM stop");
        assert!(
            matches!(
                error.code,
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                    | fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
            ),
            "{error:?}"
        );
    } else {
        outcome.expect("network program");
    }
    assert_eq!(
        jobs, 0,
        "run must join native socket jobs before returning: {shutdown_requested}"
    );
}

#[test]
fn one_pool_worker_runs_accept_reads_writes_and_nested_callbacks() {
    let source = include_str!("fixtures/pool_progress.fpas");
    for config in [
        CollectionSchedule::default(),
        CollectionSchedule {
            allocations: NonZeroUsize::new(1).unwrap(),
            mutations: NonZeroUsize::new(1).unwrap(),
            safe_points: NonZeroUsize::new(1).unwrap(),
        },
    ] {
        for cancellable in [false, true] {
            run(
                source,
                if cancellable {
                    vec!["cancellable".into()]
                } else {
                    Vec::new()
                },
                config,
            );
        }
    }
}

#[test]
fn one_pool_worker_drains_a_sender_larger_than_the_socket_buffers() {
    run(
        include_str!("fixtures/pool_write.fpas"),
        Vec::new(),
        CollectionSchedule::default(),
    );
}

#[test]
fn one_pool_worker_cancels_closes_and_times_out_pending_socket_calls() {
    for mode in [
        "read_cancel",
        "accept_cancel",
        "accept_close",
        "timeout",
        "scope_exit",
        "vm_shutdown",
    ] {
        run(
            include_str!("fixtures/pool_control.fpas"),
            vec![mode.into()],
            CollectionSchedule::default(),
        );
    }
}

#[test]
fn one_pool_worker_runs_a_tls_peer_while_connect_and_nested_callbacks_wait() {
    for config in [
        CollectionSchedule::default(),
        CollectionSchedule {
            allocations: NonZeroUsize::new(1).unwrap(),
            mutations: NonZeroUsize::new(1).unwrap(),
            safe_points: NonZeroUsize::new(1).unwrap(),
        },
    ] {
        for mode in ["tls_cancel", "tls_timeout", "scope_exit", "vm_shutdown"] {
            run(
                include_str!("fixtures/pool_connect.fpas"),
                vec![mode.into()],
                config,
            );
        }
    }
}

#[test]
fn all_stop_debugger_keeps_connects_peers_and_nested_callbacks_runnable() {
    use crate::execution::vm::debug::{DebugRunResult, DebugSession};

    for (source, mode) in [
        (include_str!("fixtures/pool_progress.fpas"), ""),
        (include_str!("fixtures/pool_progress.fpas"), "cancellable"),
        (include_str!("fixtures/pool_connect.fpas"), "tls_cancel"),
        (include_str!("fixtures/pool_connect.fpas"), "tls_timeout"),
        (include_str!("fixtures/pool_connect.fpas"), "scope_exit"),
    ] {
        let (program, errors) = fpas_parser::parse(source);
        assert!(errors.is_empty(), "{errors:?}");
        let image = fpas_compiler::compile(&program).unwrap();
        let mut session = DebugSession::with_args(
            image,
            if mode.is_empty() {
                Vec::new()
            } else {
                vec![mode.into()]
            },
        )
        .unwrap();
        assert!(
            matches!(
                session.continue_execution().unwrap(),
                DebugRunResult::Terminated(_)
            ),
            "debugger network fixture stopped: {mode}"
        );
    }
}
