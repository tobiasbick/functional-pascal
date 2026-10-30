//! Native entropy buffers, suspended cyclic callbacks and MCP executors are fully reclaimed.

use crate::{
    measurement::completed_threads,
    workloads::{image, schedule},
};
use fpas_runtime::{
    CollectionSchedule,
    execution::{ShutdownHandle, Vm},
};
use std::{
    io::{self, Write},
    sync::{Arc, OnceLock, Weak},
};

struct Writer(Arc<OnceLock<Weak<ShutdownHandle>>>, bool);
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.1 {
            self.0.get().unwrap().upgrade().unwrap().shutdown();
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn entropy() {
    let executable = image(include_str!(
        "../../src/execution/vm/hosted/entropy/fixtures/pool.fpas"
    ));
    for config in [CollectionSchedule::default(), schedule()] {
        for kind in ["bytes", "integer", "randomize", "automatic"] {
            for mode in ["normal", "scope_exit", "vm_shutdown"] {
                completed_threads(&format!("native entropy {kind} {mode}"), 4, || {
                    let slot = Arc::new(OnceLock::new());
                    let mut vm = Vm::with_writer_and_args(
                        executable.clone(),
                        Box::new(Writer(slot.clone(), mode == "vm_shutdown")),
                        vec![mode.into(), kind.into()],
                    )
                    .with_collection_schedule(config);
                    let stop = Arc::new(vm.shutdown_handle());
                    assert!(slot.set(Arc::downgrade(&stop)).is_ok());
                    let result = vm.run();
                    if mode == "vm_shutdown" {
                        assert!(matches!(
                            result.unwrap_err().code,
                            fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                                | fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
                        ));
                    } else {
                        result.unwrap();
                    }
                    drop(vm);
                    drop(stop);
                });
            }
        }
    }
}

pub fn mcp() {
    // A feature-unified Tokio build may retain its process-global Unix signal
    // registry after the first I/O runtime. Initialize only that runtime machinery,
    // without an MCP server or FPAS session, before exact per-VM measurements.
    std::thread::spawn(|| {
        drop(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        );
    })
    .join()
    .unwrap();
    let metadata = crate::measurement::Footprint::now();
    std::thread::spawn(|| {
        drop(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        );
    })
    .join()
    .unwrap();
    assert_eq!(
        crate::measurement::Footprint::now(),
        metadata,
        "Tokio runtime machinery must not keep growing"
    );
    let executable = image(include_str!(
        "../../src/execution/vm/hosted/mcp/fixtures/publication.fpas"
    ));
    for config in [CollectionSchedule::default(), schedule()] {
        for mode in ["normal", "scope_exit"] {
            completed_threads(&format!("MCP initial executor {mode}"), 4, || {
                Vm::with_args(executable.clone(), vec![mode.into()])
                    .with_collection_schedule(config)
                    .run()
                    .unwrap();
            });
        }
    }
}
