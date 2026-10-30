//! A second stop request cannot overtake retirement of an extracted native wait.

use super::*;
use crate::{
    ConsoleIntrinsic, Intrinsic, Value,
    execution::vm::{tasks::TaskScheduler, worker::Worker},
};
use std::{
    io::{self, Write},
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

struct Writer {
    entered: mpsc::Sender<()>,
    released: mpsc::Receiver<()>,
}
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.entered.send(()).unwrap();
        self.released
            .recv_timeout(Duration::from_secs(10))
            .map_err(io::Error::other)?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn concurrent_shutdown_waits_for_native_retirement_before_the_final_join_barrier() {
    let (program, errors) =
        fpas_parser::parse("program S; uses Std.Console as C; begin C.WriteText('hold'); end.");
    assert!(errors.is_empty(), "{errors:?}");
    let (entered, entry) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let hosted = Arc::new(HostedState::new(
        crate::stdlib::Console::with_writer(Box::new(Writer { entered, released })),
        vec![],
    ));
    let scheduler = Arc::new(TaskScheduler::new());
    let mut worker = Worker::new(Arc::new(fpas_compiler::compile(&program).unwrap()))
        .unwrap()
        .with_scheduler(Some(scheduler.clone()));
    worker.hosted = hosted.clone();
    worker.task_id = 7;
    worker.retain_result = true;
    scheduler.register_result(7);
    worker
        .start_console_io(
            Intrinsic::Console(ConsoleIntrinsic::Write),
            &[Value::Str("hold".into())],
            None,
        )
        .unwrap();
    entry.recv_timeout(Duration::from_secs(5)).unwrap();
    scheduler.enqueue(worker.take_task_state());

    // Hold retirement after queue extraction, while permitting another stop caller.
    let retirement = hosted.retired_native_jobs.lock().unwrap();
    let first = scheduler.clone();
    let a = std::thread::spawn(move || first.finish_main());
    let deadline = Instant::now() + Duration::from_secs(5);
    while scheduler.has_queued() && Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(
        !scheduler.has_queued(),
        "first stop did not extract queued work"
    );
    let second = scheduler.clone();
    let (done, completed) = mpsc::channel();
    let b = std::thread::spawn(move || {
        second.finish_main();
        done.send(()).unwrap();
    });
    let overtook_retirement = completed.recv_timeout(Duration::from_millis(100)).is_ok();
    drop(retirement);
    release.send(()).unwrap();
    a.join().unwrap();
    b.join().unwrap();
    hosted.wait_process_handles();
    assert!(
        !overtook_retirement,
        "a stop caller returned before saved native work was retired"
    );
    assert_eq!(
        hosted
            .console_jobs
            .load(std::sync::atomic::Ordering::Acquire),
        0
    );
    assert!(hosted.retired_native_jobs.lock().unwrap().is_empty());
}
