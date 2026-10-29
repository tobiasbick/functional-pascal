//! Exact allocation balances for file jobs, cyclic callbacks and VM-owned descriptors.

use crate::measurement::completed_threads;
use crate::workloads::{image, schedule};
use fpas_runtime::{
    CollectionSchedule,
    execution::{ShutdownHandle, Vm},
};
use std::{
    io::{self, Write},
    path::PathBuf,
    sync::{
        Arc, OnceLock, Weak,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::Duration,
};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        loop {
            let path = std::env::temp_dir().join(format!(
                "fpas-gc-file-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(file) => {
                    drop(file);
                    return Self(path);
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("file memory fixture: {error}"),
            }
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

struct Output(Arc<OnceLock<Weak<ShutdownHandle>>>);

impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        assert_eq!(bytes, b"pending");
        self.0.get().unwrap().upgrade().unwrap().shutdown();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn file_job_teardown() {
    let executable = image(include_str!(
        "../../src/execution/vm/hosted/fs/fixtures/pool.fpas"
    ));
    for config in [CollectionSchedule::default(), schedule()] {
        for mode in ["normal", "cancel", "scope_exit", "vm_shutdown"] {
            let fixture = Fixture::new();
            let path = fixture.0.to_string_lossy().into_owned();
            completed_threads(&format!("native file teardown {mode}"), 4, || {
                let stop_slot = Arc::new(OnceLock::new());
                let mut vm = Vm::with_writer_and_args(
                    executable.clone(),
                    Box::new(Output(stop_slot.clone())),
                    vec![path.clone(), mode.into()],
                )
                .with_collection_schedule(config);
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
                    stop.shutdown();
                }
                let result = outcome.expect("file VM must finish and join its native jobs");
                running.join().unwrap();
                if mode == "vm_shutdown" {
                    let error = result.expect_err("explicit stop");
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
