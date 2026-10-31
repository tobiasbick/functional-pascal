//! Host terminal providers and process startup cannot retain the sole VM worker.

use crate::{
    CollectionSchedule,
    execution::{InteractiveProcessStdio, InteractiveProcessTerminal, vm::Vm},
};
use std::{
    io::{self, Write},
    process::Stdio,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

struct Release {
    entered: mpsc::Receiver<()>,
    release: mpsc::Sender<()>,
    stop: Option<Arc<std::sync::OnceLock<std::sync::Weak<crate::execution::vm::ShutdownHandle>>>>,
}

impl Write for Release {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        assert_eq!(bytes, b"release");
        self.entered
            .recv_timeout(Duration::from_secs(5))
            .map_err(io::Error::other)?;
        if let Some(stop) = &self.stop {
            stop.get().unwrap().upgrade().unwrap().shutdown();
        }
        self.release.send(()).map_err(io::Error::other)?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn terminal_provider_and_process_handles_release_one_worker_inside_callbacks() {
    let (program, errors) = fpas_parser::parse(include_str!("fixtures/native.fpas"));
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
        let (entered, receipt) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let released = Arc::new(Mutex::new(released));
        let fallback = release.clone();
        let mut vm = Vm::with_writer_and_args(
            image.clone(),
            Box::new(Release {
                entered: receipt,
                release,
                stop: None,
            }),
            vec![
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
        let stop = vm.shutdown_handle();
        let (send, receive) = mpsc::channel();
        let running = std::thread::spawn(move || {
            let outcome = vm.run();
            drop(vm);
            let _ = send.send(outcome);
        });
        let result = receive.recv_timeout(Duration::from_secs(30));
        if result.is_err() {
            stop.shutdown();
            let _ = fallback.send(());
        }
        running.join().unwrap();
        result
            .expect("native terminal opening must release the only VM worker")
            .unwrap();
        drop(stop);
        assert!(hosted.upgrade().is_none());
    }
}

#[test]
fn debugger_runs_terminal_processes_and_process_handle_cleanup() {
    use crate::execution::vm::debug::{DebugRunResult, DebugSession};
    let source = include_str!("fixtures/native.fpas");
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty());
    let mut debug = DebugSession::with_args(
        fpas_compiler::compile(&program).unwrap(),
        vec![
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ],
    )
    .unwrap();
    debug
        .set_interactive_process_terminal(InteractiveProcessTerminal::new(|| {
            Ok(InteractiveProcessStdio::new(
                Stdio::null(),
                Stdio::null(),
                Stdio::null(),
            ))
        }))
        .unwrap();
    assert!(matches!(
        debug.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn shutdown_from_a_host_writer_returns_before_joining_a_blocked_terminal_provider() {
    use std::sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    };
    let (program, errors) = fpas_parser::parse(include_str!("fixtures/native.fpas"));
    assert!(errors.is_empty());
    let (entered, receipt) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let fallback = release.clone();
    let released = Mutex::new(released);
    let slot = Arc::new(OnceLock::new());
    let returned = Arc::new(AtomicBool::new(false));
    let acknowledgement = returned.clone();
    let mut vm = Vm::with_writer_and_args(
        fpas_compiler::compile(&program).unwrap(),
        Box::new(Release {
            entered: receipt,
            release,
            stop: Some(slot.clone()),
        }),
        vec![
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ],
    );
    vm.pool_size = 1;
    vm.set_interactive_process_terminal(InteractiveProcessTerminal::new(move || {
        entered.send(()).map_err(io::Error::other)?;
        released
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(10))
            .map_err(io::Error::other)?;
        acknowledgement.store(true, Ordering::Release);
        Ok(InteractiveProcessStdio::new(
            Stdio::null(),
            Stdio::null(),
            Stdio::null(),
        ))
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
    let error = result
        .unwrap()
        .expect_err("host callback requested shutdown");
    assert!(
        matches!(
            error.code,
            fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                | fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
        ),
        "{error:?}"
    );
    assert!(
        returned.load(Ordering::Acquire),
        "shutdown must return so the writer can release the provider"
    );
    assert_eq!(hosted.process_jobs.load(Ordering::Acquire), 0);
    let weak_hosted = Arc::downgrade(&hosted);
    drop(hosted);
    drop(stop);
    assert!(weak_hosted.upgrade().is_none());
}
