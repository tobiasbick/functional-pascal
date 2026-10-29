//! Configuration waits must release one worker and retire their native resources.

use super::super::HostedState;
use crate::execution::vm::{
    ShutdownHandle, Vm,
    debug::{DebugRunResult, DebugSession},
};
use crate::{CollectionSchedule, VerifiedExecutable};
use std::{
    io::{self, Write},
    sync::{Arc, OnceLock, Weak, atomic::Ordering, mpsc},
    time::{Duration, Instant},
};

const SOURCE: &str = include_str!("fixtures/pool_config.fpas");

pub(super) struct Fixture(std::path::PathBuf);

impl Fixture {
    pub(super) fn new() -> Self {
        use std::sync::atomic::AtomicU64;
        static NEXT: AtomicU64 = AtomicU64::new(1);
        loop {
            let path = std::env::temp_dir().join(format!(
                "fpas-net-config-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("native configuration fixture: {error}"),
            }
        }
    }
    pub(super) fn path(&self, name: &str) -> String {
        self.0.join(name).to_string_lossy().into_owned()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Output {
    hosted: Arc<OnceLock<Weak<HostedState>>>,
    stop: Arc<OnceLock<Weak<ShutdownHandle>>>,
    shutdown: bool,
}

impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let state = self.hosted.get().unwrap().upgrade().unwrap();
        match bytes {
            b"locked" => {
                let deadline = Instant::now() + Duration::from_secs(5);
                while !state.network_connections.has_busy_connection() {
                    assert!(
                        Instant::now() < deadline,
                        "reader did not acquire its native lock"
                    );
                    std::thread::park_timeout(Duration::from_millis(1));
                }
            }
            b"pending" => {
                assert!(
                    state.network_jobs.load(Ordering::Acquire) >= 2,
                    "the read and timeout configuration must both be pending"
                );
                if self.shutdown {
                    self.stop.get().unwrap().upgrade().unwrap().shutdown();
                }
            }
            _ => panic!("unexpected configuration fixture output"),
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn image(source: &str) -> VerifiedExecutable {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program).unwrap()
}

fn run<R>(
    image: &VerifiedExecutable,
    args: Vec<String>,
    config: CollectionSchedule,
    rescue: impl FnOnce() -> R,
) {
    let hosted = Arc::new(OnceLock::new());
    let stop_slot = Arc::new(OnceLock::new());
    let shutdown = args.first().is_some_and(|mode| mode == "vm_shutdown");
    let mut vm = Vm::with_writer_and_args(
        image.clone(),
        Box::new(Output {
            hosted: hosted.clone(),
            stop: stop_slot.clone(),
            shutdown,
        }),
        args,
    )
    .with_collection_schedule(config);
    vm.pool_size = 1;
    let weak = Arc::downgrade(&vm.hosted);
    assert!(hosted.set(weak.clone()).is_ok());
    let stop = Arc::new(vm.shutdown_handle());
    assert!(stop_slot.set(Arc::downgrade(&stop)).is_ok());
    let (send, receive) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        let result = vm.run();
        drop(vm);
        let _ = send.send(result);
    });
    let result = receive.recv_timeout(Duration::from_secs(30));
    let _rescue = if result.is_err() {
        stop.shutdown();
        Some(rescue())
    } else {
        None
    };
    thread.join().unwrap();
    let result = result.expect("configuration waits must release the only VM worker");
    if shutdown {
        let error = result.expect_err("explicit VM stop");
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
    assert!(
        weak.upgrade().is_none(),
        "configuration wait retained its VM"
    );
}

#[test]
fn busy_timeout_configuration_releases_one_worker_and_cleans_cyclic_callbacks() {
    use std::num::NonZeroUsize;
    let image = image(SOURCE);
    for config in [
        CollectionSchedule::default(),
        CollectionSchedule {
            allocations: NonZeroUsize::new(1).unwrap(),
            mutations: NonZeroUsize::new(1).unwrap(),
            safe_points: NonZeroUsize::new(1).unwrap(),
        },
    ] {
        for mode in ["normal", "scope_exit", "vm_shutdown"] {
            run(&image, vec![mode.into()], config, || {});
        }
    }
}

#[cfg(unix)]
#[test]
fn tls_certificate_loading_releases_one_worker_and_the_debugger() {
    use std::os::unix::fs::OpenOptionsExt;
    let image = image(include_str!("fixtures/pool_tls_config.fpas"));
    let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    for debug in [false, true] {
        let fixture = Fixture::new();
        let path = fixture.path("certificate.fifo");
        let key = fixture.path("key.pem");
        std::fs::write(&key, certificate.signing_key.serialize_pem()).unwrap();
        nix::unistd::mkfifo(
            std::path::Path::new(&path),
            nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
        )
        .unwrap();
        let args = vec![path.clone(), key, certificate.cert.pem()];
        if debug {
            let mut session = DebugSession::with_args(image.clone(), args).unwrap();
            assert!(matches!(
                session.continue_execution().unwrap(),
                DebugRunResult::Terminated(_)
            ));
        } else {
            run(&image, args, CollectionSchedule::default(), || {
                let read = std::fs::OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NONBLOCK)
                    .open(&path)
                    .unwrap();
                if let Ok(mut write) = std::fs::OpenOptions::new()
                    .write(true)
                    .custom_flags(libc::O_NONBLOCK)
                    .open(&path)
                {
                    let _ = write.write_all(b"abort");
                }
                read
            });
        }
    }
}
