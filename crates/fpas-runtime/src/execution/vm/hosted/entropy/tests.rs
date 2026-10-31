//! Controlled entropy requires another task to release the first OS call on one worker.

use crate::{
    CollectionSchedule,
    execution::vm::{ShutdownHandle, Vm},
};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex, OnceLock, Weak, mpsc},
    time::Duration,
};

struct Writer {
    entered: mpsc::Receiver<()>,
    release: mpsc::Sender<()>,
    stop: Arc<OnceLock<Weak<ShutdownHandle>>>,
    shutdown: bool,
}
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        assert_eq!(bytes, b"release");
        self.entered
            .recv_timeout(Duration::from_secs(5))
            .map_err(io::Error::other)?;
        if self.shutdown {
            self.stop.get().unwrap().upgrade().unwrap().shutdown();
        }
        self.release.send(()).map_err(io::Error::other)?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn entropy_releases_one_worker_and_joins_callback_shutdown_and_scope_exit() {
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
        for kind in ["bytes", "integer", "randomize", "automatic"] {
            for mode in ["normal", "scope_exit", "vm_shutdown"] {
                let (entered, entry) = mpsc::channel();
                let (release, released) = mpsc::channel();
                let fallback = release.clone();
                let released = Mutex::new(Some(released));
                let slot = Arc::new(OnceLock::new());
                let mut vm = Vm::with_writer_and_args(
                    image.clone(),
                    Box::new(Writer {
                        entered: entry,
                        release,
                        stop: slot.clone(),
                        shutdown: mode == "vm_shutdown",
                    }),
                    vec![mode.into(), kind.into()],
                )
                .with_collection_schedule(config);
                vm.pool_size = 1;
                *vm.hosted.entropy_source.provider.lock().unwrap() = Some(Arc::new(move |bytes| {
                    if let Some(released) = released.lock().unwrap().take() {
                        entered.send(()).unwrap();
                        released.recv_timeout(Duration::from_secs(10)).unwrap();
                    }
                    bytes.fill(17);
                    Ok(())
                }));
                let hosted = Arc::clone(&vm.hosted);
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
                    assert!(matches!(
                        result.unwrap_err().code,
                        fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                            | fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
                    ));
                } else {
                    result.unwrap();
                }
                assert_eq!(
                    hosted
                        .entropy_jobs
                        .load(std::sync::atomic::Ordering::Acquire),
                    0
                );
                let weak_hosted = Arc::downgrade(&hosted);
                drop(hosted);
                drop(stop);
                assert!(weak_hosted.upgrade().is_none());
            }
        }
    }
}

#[test]
fn all_stop_debugger_resumes_entropy_and_seeded_calls_inside_callbacks() {
    use crate::execution::vm::debug::{DebugRunResult, DebugSession};
    let (program, errors) = fpas_parser::parse(include_str!("fixtures/pool.fpas"));
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    for kind in ["bytes", "integer", "randomize", "automatic"] {
        let mut debug =
            DebugSession::with_args(image.clone(), vec!["normal".into(), kind.into()]).unwrap();
        assert!(matches!(
            debug.continue_execution().unwrap(),
            DebugRunResult::Terminated(_)
        ));
    }
}

#[test]
fn unavailable_entropy_preserves_typed_crypto_and_random_source_failures() {
    for pooled in [false, true] {
        for (operation, random) in [
            (
                "if not (Crypto.RandomBytes(8) is Error(_)) then panic('missing entropy failure'); end if;",
                false,
            ),
            (
                "if not (Crypto.RandomInt(0, 10) is Error(_)) then panic('missing entropy failure'); end if;",
                false,
            ),
            ("Random.Randomize();", true),
            ("discard Random.Random();", true),
            ("discard Random.RandomInt(0, 10);", true),
        ] {
            let body = if pooled {
                "scope const Pending := go Draw(); Tasks.Wait(Pending); end scope;"
            } else {
                "Draw();"
            };
            let source = format!(
                "program Unavailable; uses Std.Crypto as Crypto, Std.Math.Random as Random, Std.Tasks as Tasks; procedure Draw(); begin {operation} end procedure; begin {body} end."
            );
            let (program, errors) = fpas_parser::parse(&source);
            assert!(errors.is_empty(), "{errors:?}");
            let image = fpas_compiler::compile(&program).unwrap();
            let mut vm = Vm::new(image);
            *vm.hosted.entropy_source.provider.lock().unwrap() =
                Some(Arc::new(|_| Err(getrandom::Error::UNSUPPORTED)));
            let result = vm.run();
            if random {
                assert_eq!(
                    result.unwrap_err().code,
                    fpas_diagnostics::codes::RUNTIME_RANDOM_SOURCE_FAILURE
                );
            } else {
                result.unwrap();
            }
        }
    }
}
