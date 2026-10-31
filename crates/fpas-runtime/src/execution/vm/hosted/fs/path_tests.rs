//! Whole-file/path operations suspend callbacks and release a single VM worker.

use super::pool_tests::{Gate, Output};
use super::tests::Fixture;
use crate::execution::vm::{
    Vm,
    debug::{DebugRunResult, DebugSession},
};
use crate::{CollectionSchedule, VerifiedExecutable};
use std::{
    sync::{Arc, OnceLock, mpsc},
    time::Duration,
};

const SOURCE: &str = include_str!("fixtures/paths.fpas");

fn image() -> VerifiedExecutable {
    let (program, errors) = fpas_parser::parse(SOURCE);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program).unwrap()
}

fn run(image: &VerifiedExecutable, root: &str, mode: &str, config: CollectionSchedule) {
    let hosted_slot = Arc::new(OnceLock::new());
    let stop_slot = Arc::new(OnceLock::new());
    let gate = Arc::new(Gate::default());
    let mut vm = Vm::with_writer_and_args(
        image.clone(),
        Box::new(Output {
            hosted: hosted_slot.clone(),
            stop: stop_slot.clone(),
            gate,
        }),
        vec![root.into(), mode.into()],
    )
    .with_collection_schedule(config);
    vm.pool_size = 1;
    let hosted = Arc::downgrade(&vm.hosted);
    assert!(hosted_slot.set(hosted.clone()).is_ok());
    let stop = Arc::new(vm.shutdown_handle());
    assert!(stop_slot.set(Arc::downgrade(&stop)).is_ok());
    let (sent, received) = mpsc::channel();
    let running = std::thread::spawn(move || {
        let result = vm.run();
        drop(vm);
        let _ = sent.send(result);
    });
    let outcome = received.recv_timeout(Duration::from_secs(30));
    #[cfg(unix)]
    let _rescue = if outcome.is_err() && mode == "fifo" {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let read = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(format!("{root}/fifo"))
            .unwrap();
        if let Ok(mut write) = std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(format!("{root}/fifo"))
        {
            let _ = write.write_all(b"abort");
        }
        Some(read)
    } else {
        None
    };
    if outcome.is_err() {
        stop.shutdown();
    }
    running.join().unwrap();
    let outcome = outcome.expect("native path I/O must release the only VM worker");
    if mode == "vm_shutdown" {
        let error = outcome.expect_err("explicit stop");
        assert!(
            matches!(
                error.code,
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                    | fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
            ),
            "{error:?}"
        );
    } else {
        outcome.unwrap();
    }
    drop(stop);
    assert!(hosted.upgrade().is_none(), "path wait retained its VM");
}

#[test]
fn path_calls_and_queries_resume_inline_callbacks_and_clean_up() {
    use std::num::NonZeroUsize;
    let image = image();
    for config in [
        CollectionSchedule::default(),
        CollectionSchedule {
            allocations: NonZeroUsize::new(1).unwrap(),
            mutations: NonZeroUsize::new(1).unwrap(),
            safe_points: NonZeroUsize::new(1).unwrap(),
        },
    ] {
        for mode in ["normal", "scope_exit", "vm_shutdown"] {
            let fixture = Fixture::new();
            let root = fixture.path("paths");
            std::fs::create_dir(&root).unwrap();
            run(&image, &root, mode, config);
        }
    }
}

#[cfg(unix)]
#[test]
fn fifo_read_and_write_release_one_worker_and_the_debugger() {
    let image = image();
    for debug in [false, true] {
        let fixture = Fixture::new();
        let root = fixture.path("paths");
        std::fs::create_dir(&root).unwrap();
        nix::unistd::mkfifo(
            std::path::Path::new(&root).join("fifo").as_path(),
            nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
        )
        .unwrap();
        if debug {
            let mut session =
                DebugSession::with_args(image.clone(), vec![root, "fifo".into()]).unwrap();
            assert!(matches!(
                session.continue_execution().unwrap(),
                DebugRunResult::Terminated(_)
            ));
        } else {
            run(&image, &root, "fifo", CollectionSchedule::default());
        }
    }
}

#[test]
fn debugger_path_calls_preserve_native_results_and_scope_cleanup() {
    let image = image();
    for mode in ["normal", "scope_exit"] {
        let fixture = Fixture::new();
        let root = fixture.path("paths");
        std::fs::create_dir(&root).unwrap();
        let mut session = DebugSession::with_args(image.clone(), vec![root, mode.into()]).unwrap();
        let result = session.continue_execution().unwrap();
        assert!(
            matches!(result, DebugRunResult::Terminated(_)),
            "mode {mode}: {result:?}"
        );
    }
}
