//! Real native gates prove progress with one VM worker and root-free job teardown.

use super::*;
use crate::{CollectionSchedule, execution::vm::Vm};
use std::{
    io::{self, Read, Write},
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

struct Writer {
    entered: mpsc::Sender<()>,
    released: mpsc::Receiver<()>,
    output: Arc<Mutex<Vec<u8>>>,
    stop: Arc<OnceLock<Weak<crate::execution::vm::ShutdownHandle>>>,
    shutdown: bool,
    first: bool,
}

impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.first {
            self.first = false;
            assert_eq!(bytes, b"hold");
            self.entered.send(()).map_err(io::Error::other)?;
            self.released
                .recv_timeout(Duration::from_secs(10))
                .map_err(io::Error::other)?;
            if self.shutdown {
                self.stop.get().unwrap().upgrade().unwrap().shutdown();
            }
        }
        self.output.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct Reader {
    entered: mpsc::Receiver<()>,
    release: mpsc::Sender<()>,
    bytes: io::Cursor<Vec<u8>>,
    first: bool,
}

impl Read for Reader {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if self.first {
            self.first = false;
            self.entered
                .recv_timeout(Duration::from_secs(5))
                .map_err(io::Error::other)?;
            self.release.send(()).map_err(io::Error::other)?;
        }
        self.bytes.read(bytes)
    }
}

#[test]
fn console_writes_and_text_reads_release_one_worker_and_preserve_frames() {
    let (program, errors) = fpas_parser::parse(include_str!("fixtures/pool.fpas"));
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    for config in [
        CollectionSchedule::default(),
        CollectionSchedule {
            allocations: std::num::NonZeroUsize::new(1).unwrap(),
            mutations: std::num::NonZeroUsize::new(1).unwrap(),
            safe_points: std::num::NonZeroUsize::new(1).unwrap(),
        },
    ] {
        for mode in ["normal", "scope_exit", "vm_shutdown"] {
            let (entered, entry) = mpsc::channel();
            let (release, released) = mpsc::channel();
            let fallback = release.clone();
            let output = Arc::new(Mutex::new(Vec::new()));
            let slot = Arc::new(OnceLock::new());
            let mut vm = Vm::with_writer_and_args(
                image.clone(),
                Box::new(Writer {
                    entered,
                    released,
                    output: output.clone(),
                    stop: slot.clone(),
                    shutdown: mode == "vm_shutdown",
                    first: true,
                }),
                vec![mode.into()],
            )
            .with_collection_schedule(config);
            vm.pool_size = 1;
            vm.set_text_input_reader(io::BufReader::new(Reader {
                entered: entry,
                release,
                bytes: io::Cursor::new("line🙂\r\nlast".as_bytes().to_vec()),
                first: true,
            }));
            let hosted = Arc::downgrade(&vm.hosted);
            let stop = Arc::new(vm.shutdown_handle());
            assert!(slot.set(Arc::downgrade(&stop)).is_ok());
            let (send, receive) = mpsc::channel();
            let thread = std::thread::spawn(move || {
                let result = vm.run();
                drop(vm);
                let _ = send.send(result);
            });
            let result = receive.recv_timeout(Duration::from_secs(30));
            if result.is_err() {
                let _ = fallback.send(());
                stop.shutdown();
            }
            thread.join().unwrap();
            let result = result.unwrap();
            if mode == "vm_shutdown" {
                let error = result.expect_err("console writer requested shutdown");
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
            if mode == "normal" {
                assert!(output.lock().unwrap().starts_with(b"hold65536\n"));
            }
            assert_eq!(
                hosted
                    .upgrade()
                    .unwrap()
                    .console_jobs
                    .load(Ordering::Acquire),
                0
            );
            drop(stop);
            assert!(hosted.upgrade().is_none(), "console jobs retained their VM");
        }
    }
}

#[test]
fn cancellation_while_waiting_for_a_console_lock_does_not_write() {
    let state = Arc::new(HostedState::new(Console::new(), Vec::new()));
    let guard = state.console.lock().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    let owned = state.clone();
    let (entered, entry) = mpsc::channel();
    let (sent, received) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        let first = AtomicBool::new(true);
        let call = request::Call {
            operation: crate::ConsoleIntrinsic::Write,
            numbers: [0; 4],
            payload: request::Payload::Text("lost".into()),
            location: SourceLocation::new(1, 1),
        };
        let result = call.run(&owned, || {
            if first.swap(false, Ordering::AcqRel) {
                entered.send(()).unwrap();
            }
            stopped.load(Ordering::Acquire)
        });
        let _ = sent.send(result);
    });
    entry.recv_timeout(Duration::from_secs(5)).unwrap();
    stop.store(true, Ordering::Release);
    let result = received.recv_timeout(Duration::from_secs(5));
    drop(guard);
    thread.join().unwrap();
    assert_eq!(
        result.unwrap().err().unwrap().code,
        fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
    );
    assert!(state.console.lock().unwrap().output().lines.is_empty());
}

#[test]
fn all_stop_debugger_resumes_console_callbacks_frames_and_queued_input() {
    use crate::execution::vm::debug::{DebugRunResult, DebugSession};
    let (program, errors) = fpas_parser::parse(include_str!("fixtures/pool.fpas"));
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    for mode in ["normal", "scope_exit"] {
        let mut debug = DebugSession::with_args(image.clone(), vec![mode.into()]).unwrap();
        debug.push_debuggee_input("line🙂").unwrap();
        debug.push_debuggee_input("last").unwrap();
        debug.signal_debuggee_eof().unwrap();
        assert!(matches!(
            debug.continue_execution().unwrap(),
            DebugRunResult::Terminated(_)
        ));
    }
}
