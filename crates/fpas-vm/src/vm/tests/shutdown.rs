//! VM teardown joins real idle workers and the timer driver.

use std::sync::mpsc;
use std::time::Duration;

use super::{return_unit, verified};
use crate::vm::Vm;

#[test]
fn repeated_short_runs_join_idle_workers_and_the_timer_driver() {
    let executable = verified(
        vec![return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        1,
    );
    let (finished_tx, finished_rx) = mpsc::channel();
    let runner = std::thread::spawn(move || {
        for _ in 0..64 {
            let mut vm = Vm::new(executable.clone());
            vm.pool_size = 1;
            vm.run().expect("short pooled run should succeed");
        }
        finished_tx.send(()).unwrap();
    });

    finished_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("idle workers and timer must leave during teardown");
    runner.join().unwrap();
}
