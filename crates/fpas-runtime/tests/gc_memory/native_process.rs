//! Terminal provider gates retain cyclic callbacks without retaining VM allocations.

use crate::{
    measurement::completed_threads,
    workloads::{image, schedule},
};
use fpas_runtime::{
    CollectionSchedule,
    execution::{InteractiveProcessStdio, InteractiveProcessTerminal, ShutdownHandle, Vm},
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
        // A native provider cannot be forcibly interrupted. Release it before
        // shutdown joins its operation, while the callback's task is still pending.
        self.release.send(()).map_err(io::Error::other)?;
        if self.shutdown {
            self.stop.get().unwrap().upgrade().unwrap().shutdown();
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn teardown() {
    let executable = image(include_str!(
        "../../src/execution/vm/hosted/proc/fixtures/native.fpas"
    ));
    let command = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    for config in [CollectionSchedule::default(), schedule()] {
        for mode in ["normal", "scope_exit", "vm_shutdown"] {
            completed_threads(&format!("native process boundary {mode}"), 4, || {
                let (entered, receipt) = mpsc::channel();
                let (release, released) = mpsc::channel();
                let released = Mutex::new(released);
                let fallback = release.clone();
                let slot = Arc::new(OnceLock::new());
                let mut vm = Vm::with_writer_and_args(
                    executable.clone(),
                    Box::new(Output {
                        entered: receipt,
                        release,
                        stop: slot.clone(),
                        shutdown: mode == "vm_shutdown",
                    }),
                    vec![command.clone(), mode.into()],
                )
                .with_collection_schedule(config);
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
                drop(stop);
            });
        }
    }
}
