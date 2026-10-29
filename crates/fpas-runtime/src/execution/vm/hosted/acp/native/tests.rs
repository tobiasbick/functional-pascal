//! A gated authentication terminal requires another task on the sole VM worker.

use crate::execution::vm::{ShutdownHandle, Vm};
use crate::{
    CollectionSchedule,
    execution::{InteractiveProcessStdio, InteractiveProcessTerminal},
};
use std::{
    io::{self, Write},
    process::Stdio,
    sync::{Arc, Mutex, OnceLock, Weak, mpsc},
    time::Duration,
};

struct Output {
    entered: mpsc::Receiver<()>,
    release: mpsc::Sender<()>,
    stop: Arc<OnceLock<Weak<ShutdownHandle>>>,
    shutdown: bool,
}
impl Write for Output {
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
fn authentication_releases_one_worker_and_joins_on_scope_exit_and_callback_shutdown() {
    let (program, errors) = fpas_parser::parse(include_str!("fixtures/auth.fpas"));
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
            let (entered, receipt) = mpsc::channel();
            let (release, released) = mpsc::channel();
            let fallback = release.clone();
            let released = Mutex::new(released);
            let slot = Arc::new(OnceLock::new());
            let mut vm = Vm::with_writer_and_args(
                image.clone(),
                Box::new(Output {
                    entered: receipt,
                    release,
                    stop: slot.clone(),
                    shutdown: mode == "vm_shutdown",
                }),
                vec![
                    mode.into(),
                    std::env::current_exe()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                ],
            )
            .with_collection_schedule(config);
            vm.pool_size = 1;
            vm.set_interactive_process_terminal(InteractiveProcessTerminal::new(move || {
                entered.send(()).map_err(io::Error::other)?;
                released
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(10))
                    .map_err(io::Error::other)?;
                Ok(InteractiveProcessStdio::new(
                    Stdio::null(),
                    Stdio::null(),
                    Stdio::null(),
                ))
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
                    .upgrade()
                    .unwrap()
                    .acp_jobs
                    .load(std::sync::atomic::Ordering::Acquire),
                0
            );
            drop(stop);
            assert!(hosted.upgrade().is_none());
        }
    }
}

#[cfg(unix)]
#[test]
fn host_file_calls_release_one_worker_and_the_all_stop_debugger() {
    use crate::execution::vm::debug::{DebugRunResult, DebugSession};
    let (program, errors) = fpas_parser::parse(include_str!("fixtures/files.fpas"));
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    for debug in [false, true] {
        for mode in ["read", "write"] {
            let root = Directory::new();
            let path = root.0.join("fifo");
            nix::unistd::mkfifo(
                &path,
                nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
            )
            .unwrap();
            let args = vec![path.to_string_lossy().into_owned(), mode.into()];
            if debug {
                let mut session = DebugSession::with_args(image.clone(), args).unwrap();
                assert!(matches!(
                    session.continue_execution().unwrap(),
                    DebugRunResult::Terminated(_)
                ));
            } else {
                let mut vm = Vm::with_args(image.clone(), args);
                vm.pool_size = 1;
                vm.run().unwrap();
            }
        }
    }
}

#[cfg(unix)]
struct Directory(std::path::PathBuf);
#[cfg(unix)]
impl Directory {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        loop {
            let path = std::env::temp_dir().join(format!(
                "fpas-acp-native-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create native ACP fixture: {error}"),
            }
        }
    }
}
#[cfg(unix)]
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
