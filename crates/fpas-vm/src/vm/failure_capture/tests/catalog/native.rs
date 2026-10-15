//! A separate process proves bounded report availability while host work remains blocked.

use super::*;
use crate::vm::hosted::HostedState;
use std::{
    io::{self, BufRead, Read, Write},
    process::{Child, Command, Stdio},
    sync::{Mutex, mpsc},
    time::{Duration, Instant},
};

struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn native_peer_has_a_deadline_without_delaying_report_availability() {
    let mut child = ChildGuard(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "vm::failure_capture::tests::catalog::native::blocked_native_child",
                "--ignored",
                "--nocapture",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let (ready_tx, ready_rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in io::BufReader::new(stdout).lines() {
            if line.unwrap().contains("CAPTURE_READY_BEFORE_RELEASE") {
                ready_tx.send(()).unwrap();
            }
        }
    });
    ready_rx
        .recv_timeout(Duration::from_secs(4))
        .expect("capture must be available before host release");
    assert!(
        child.0.try_wait().unwrap().is_none(),
        "host work must still be blocked at report delivery"
    );
    child.0.stdin.take().unwrap().write_all(b"x").unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        assert!(Instant::now() < deadline, "child cleanup exceeded deadline");
        std::thread::sleep(Duration::from_millis(10));
    }
    reader.join().unwrap();
}

#[test]
#[ignore = "isolated child fixture, launched by native_peer_has_a_deadline_without_delaying_report_availability"]
fn blocked_native_child() {
    struct Blocked {
        entered: mpsc::Sender<()>,
        release: mpsc::Receiver<()>,
    }
    impl Write for Blocked {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.entered.send(()).unwrap();
            self.release
                .recv_timeout(Duration::from_secs(8))
                .map_err(io::Error::other)?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let mut vm = vm(
        "program Native; uses Std.Console as Console; procedure Background(); begin Console.WriteText('blocked'); end procedure; procedure Barrier(); begin end procedure; begin panic('root'); end.",
    );
    vm.pool_size = 1;
    let delivered = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let receipt = Arc::clone(&delivered);
    vm.hosted = Arc::new(HostedState::new(
        fpas_std::Console::with_writer(Box::new(Blocked {
            entered: entered_tx,
            release: release_rx,
        })),
        vec![],
    ));
    let task_id = vm.scheduler.alloc_id();
    let release = Mutex::new(Some(release_tx));
    assert!(vm.set_failure_observer(move |data| {
        let catalog = data.task_catalog.as_ref().unwrap();
        let peer = catalog
            .tasks
            .iter()
            .find(|task| task.id == task_id)
            .unwrap();
        assert_eq!(peer.state, "running");
        assert_eq!(peer.inspection, "unavailable");
        assert_eq!(peer.reason, Some("collection_deadline"));
        assert!(peer.snapshot.is_none());
        receipt.store(true, std::sync::atomic::Ordering::Release);
        println!("CAPTURE_READY_BEFORE_RELEASE");
        io::stdout().flush().unwrap();
        let mut byte = [0u8; 1];
        io::stdin().read_exact(&mut byte).unwrap();
        release.lock().unwrap().take().unwrap().send(()).unwrap();
    }));
    let mut background = worker(&vm, "Background", vec![]);
    background.task_id = task_id;
    background.retain_result = true;
    vm.scheduler.register_result(task_id);
    vm.scheduler.enqueue(background.take_task_state());
    let guard = start_pool(&vm);
    entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let start = Instant::now();
    vm.run().unwrap_err();
    assert!(start.elapsed() < Duration::from_secs(3));
    assert!(delivered.load(std::sync::atomic::Ordering::Acquire));
    drop(guard);
}
