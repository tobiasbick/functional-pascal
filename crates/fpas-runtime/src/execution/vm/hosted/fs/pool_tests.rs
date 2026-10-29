//! A locked native file requires another FPAS task on the same worker to release it.

use super::registry::Entry;
use super::tests::Fixture;
use crate::CollectionSchedule;
use crate::execution::vm::{ShutdownHandle, Vm, hosted::HostedState};
use std::io::{self, Write};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, OnceLock, Weak, mpsc};
use std::time::Duration;

const SOURCE: &str = include_str!("fixtures/pool.fpas");

#[derive(Default)]
pub(super) struct Gate {
    release: Mutex<Option<mpsc::Sender<()>>>,
    thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    observed_peer: std::sync::atomic::AtomicBool,
}

impl Gate {
    fn arm(&self, entry: Arc<Entry>) {
        let (release, wait) = mpsc::channel();
        let (locked, ready) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            let _held = entry.state.lock().unwrap();
            locked.send(()).unwrap();
            let _ = wait.recv();
        });
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        *self.release.lock().unwrap() = Some(release);
        *self.thread.lock().unwrap() = Some(thread);
    }

    fn release(&self) {
        if let Some(release) = self.release.lock().unwrap().take() {
            let _ = release.send(());
        }
    }

    fn armed(&self) -> bool {
        self.release.lock().unwrap().is_some()
    }

    fn join(&self) {
        self.release();
        if let Some(thread) = self.thread.lock().unwrap().take() {
            thread.join().unwrap();
        }
    }
}

pub(super) struct Output {
    pub(super) hosted: Arc<OnceLock<Weak<HostedState>>>,
    pub(super) stop: Arc<OnceLock<Weak<ShutdownHandle>>>,
    pub(super) gate: Arc<Gate>,
}

impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let hosted = self.hosted.get().unwrap().upgrade().unwrap();
        match bytes {
            b"ready" => {
                let handles = hosted.files.handles();
                assert_eq!(handles.len(), 1);
                self.gate
                    .arm(hosted.files.entry(handles[0], &mut || false).unwrap());
            }
            b"peer"
                if self.gate.armed()
                    && hosted.file_jobs.load(std::sync::atomic::Ordering::Acquire) > 0 =>
            {
                self.gate
                    .observed_peer
                    .store(true, std::sync::atomic::Ordering::Release);
                self.gate.release();
            }
            b"pending" => self.stop.get().unwrap().upgrade().unwrap().shutdown(),
            _ => {}
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn one_worker_file_calls_and_nested_callbacks_progress_and_clean_up() {
    let (program, errors) = fpas_parser::parse(SOURCE);
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    for config in [
        CollectionSchedule::default(),
        CollectionSchedule {
            allocations: NonZeroUsize::new(1).unwrap(),
            mutations: NonZeroUsize::new(1).unwrap(),
            safe_points: NonZeroUsize::new(1).unwrap(),
        },
    ] {
        for mode in ["gate", "cancel", "scope_exit", "vm_shutdown"] {
            let fixture = Fixture::new();
            let path = fixture.path("pool");
            let gate = Arc::new(Gate::default());
            let hosted_slot = Arc::new(OnceLock::new());
            let stop_slot = Arc::new(OnceLock::new());
            let mut vm = Vm::with_writer_and_args(
                image.clone(),
                Box::new(Output {
                    hosted: hosted_slot.clone(),
                    stop: stop_slot.clone(),
                    gate: gate.clone(),
                }),
                vec![path, mode.into()],
            )
            .with_collection_schedule(config);
            vm.pool_size = 1;
            let hosted = Arc::downgrade(&vm.hosted);
            assert!(hosted_slot.set(hosted.clone()).is_ok());
            let stop = Arc::new(vm.shutdown_handle());
            assert!(stop_slot.set(Arc::downgrade(&stop)).is_ok());
            let (sent, received) = mpsc::channel();
            let running = std::thread::spawn(move || {
                let outcome = vm.run();
                drop(vm);
                let _ = sent.send(outcome);
            });
            let outcome = received.recv_timeout(Duration::from_secs(30));
            if outcome.is_err() {
                gate.release();
                stop.shutdown();
            }
            let result = outcome.expect("file wait must release the only VM worker");
            running.join().unwrap();
            gate.join();
            if mode == "vm_shutdown" {
                let error = result.expect_err("VM stop");
                assert!(
                    matches!(
                        error.code,
                        fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                            | fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
                    ),
                    "{error:?}"
                );
            } else {
                result.unwrap();
            }
            if mode == "gate" {
                assert!(
                    gate.observed_peer
                        .load(std::sync::atomic::Ordering::Acquire)
                );
            }
            drop(stop);
            assert!(hosted.upgrade().is_none(), "file job retained its VM");
        }
    }
}

#[test]
fn all_stop_debugger_runs_file_calls_inside_callbacks() {
    use crate::execution::vm::debug::{DebugRunResult, DebugSession};
    let (program, errors) = fpas_parser::parse(SOURCE);
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    for mode in ["normal", "cancel", "scope_exit"] {
        let fixture = Fixture::new();
        let mut session =
            DebugSession::with_args(image.clone(), vec![fixture.path("debugger"), mode.into()])
                .unwrap();
        assert!(matches!(
            session.continue_execution().unwrap(),
            DebugRunResult::Terminated(_)
        ));
    }
}
