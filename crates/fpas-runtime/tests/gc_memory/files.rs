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

struct Fixture(PathBuf, bool);

impl Fixture {
    fn new(directory: bool) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        loop {
            let path = std::env::temp_dir().join(format!(
                "fpas-gc-file-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let created = if directory {
                std::fs::create_dir(&path)
            } else {
                std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .map(drop)
            };
            match created {
                Ok(()) => return Self(path, directory),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("file memory fixture: {error}"),
            }
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.1 {
            let _ = std::fs::remove_dir_all(&self.0);
        } else {
            let _ = std::fs::remove_file(&self.0);
        }
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
    teardown(
        include_str!("../../src/execution/vm/hosted/fs/fixtures/pool.fpas"),
        &["normal", "cancel", "scope_exit", "vm_shutdown"],
        false,
    );
}

pub fn path_job_teardown() {
    teardown(
        include_str!("../../src/execution/vm/hosted/fs/fixtures/paths.fpas"),
        &["normal", "scope_exit", "vm_shutdown"],
        true,
    );
}

fn teardown(source: &str, modes: &[&'static str], directory: bool) {
    let executable = image(source);
    for config in [CollectionSchedule::default(), schedule()] {
        for &mode in modes {
            let fixture = Fixture::new(directory);
            let path = fixture.0.to_string_lossy().into_owned();
            completed_threads(
                &format!(
                    "native {} teardown {mode}",
                    if directory { "path" } else { "file" }
                ),
                4,
                || {
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
                },
            );
        }
    }
}
