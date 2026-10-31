//! Cancellation publishes task completion only after its native wait is retired.

use super::*;
use crate::{
    ConsoleIntrinsic, Intrinsic, Value,
    execution::vm::{tasks::TaskScheduler, worker::Worker},
};
use std::{
    io::{self, Write},
    sync::{Arc, mpsc},
    time::Duration,
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
fn shutdown_does_not_publish_completion_before_native_job_is_joined() {
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
    let template = worker.pool_template();
    scheduler.enqueue(worker.take_task_state());
    scheduler.request_cancel();
    let pending_at_stop = matches!(
        scheduler.poll_result(7),
        crate::execution::vm::TaskResultPoll::Pending
    );
    let pool_scheduler = scheduler.clone();
    let (done, completed) = mpsc::channel();
    let pool = std::thread::spawn(move || {
        let result = crate::execution::vm::tasks::pool::pool_loop(&template, pool_scheduler);
        done.send(result).unwrap();
    });
    let premature = completed.recv_timeout(Duration::from_millis(100)).is_ok();
    let pending_while_running = matches!(
        scheduler.poll_result(7),
        crate::execution::vm::TaskResultPoll::Pending
    );
    release.send(()).unwrap();
    pool.join().unwrap();
    hosted.wait_process_handles();
    assert!(
        pending_at_stop,
        "cancellation must not publish before native retirement"
    );
    assert!(
        pending_while_running && !premature,
        "the blocked job must still own completion"
    );
    assert!(matches!(
        scheduler.poll_result(7),
        crate::execution::vm::TaskResultPoll::Failed(_)
    ));
    assert!(scheduler.is_shutdown());
    assert_eq!(
        hosted
            .console_jobs
            .load(std::sync::atomic::Ordering::Acquire),
        0
    );
    assert!(hosted.retired_native_jobs.lock().unwrap().is_empty());
}
