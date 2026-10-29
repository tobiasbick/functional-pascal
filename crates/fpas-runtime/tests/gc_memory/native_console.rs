//! Owned console jobs release suspended callback graphs and native buffers at teardown.

use crate::{
    measurement::completed_threads,
    workloads::{image, schedule},
};
use fpas_runtime::{
    CollectionSchedule,
    execution::{ShutdownHandle, Vm},
};
use std::{
    io::{self, Read, Write},
    sync::{Arc, OnceLock, Weak, mpsc},
    time::Duration,
};

struct Writer {
    entered: mpsc::Sender<()>,
    released: mpsc::Receiver<()>,
    stop: Arc<OnceLock<Weak<ShutdownHandle>>>,
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

pub fn teardown() {
    let executable = image(include_str!(
        "../../src/execution/vm/hosted/console_io/fixtures/pool.fpas"
    ));
    for config in [CollectionSchedule::default(), schedule()] {
        for mode in ["normal", "scope_exit", "vm_shutdown"] {
            completed_threads(&format!("native console boundary {mode}"), 4, || {
                let (entered, entry) = mpsc::channel();
                let (release, released) = mpsc::channel();
                let fallback = release.clone();
                let slot = Arc::new(OnceLock::new());
                let mut vm = Vm::with_writer_and_args(
                    executable.clone(),
                    Box::new(Writer {
                        entered,
                        released,
                        stop: slot.clone(),
                        shutdown: mode == "vm_shutdown",
                        first: true,
                    }),
                    vec![mode.into()],
                )
                .with_collection_schedule(config);
                vm.set_text_input_reader(io::BufReader::new(Reader {
                    entered: entry,
                    release,
                    bytes: io::Cursor::new("line🙂\r\nlast".as_bytes().to_vec()),
                    first: true,
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
                    let error = result.expect_err("VM stop from console writer");
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
