//! VM-owned `Std.Console` dispatch invariants.

use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use fpas_bytecode::{Chunk, ConsoleIntrinsic, Intrinsic, Op, Value};

use crate::tests::helpers::{loc, minimal_shared_state};
use crate::vm::Worker;

#[test]
fn console_delay_does_not_acquire_console_mutex() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Console(ConsoleIntrinsic::Delay))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());
    let shared = Arc::new(minimal_shared_state(chunk));
    let console = shared.console.lock().expect("console lock");
    let (done_tx, done_rx) = mpsc::channel();

    let worker_shared = Arc::clone(&shared);
    let worker = std::thread::spawn(move || {
        let mut worker = Worker::new_main(worker_shared);
        worker.push(Value::Integer(0)).expect("delay argument");
        let result = worker.run();
        done_tx.send(result).expect("send VM result");
    });

    let result = done_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("Delay(0) must complete while the console mutex is held");
    drop(console);
    worker.join().expect("worker thread");
    result.expect("Delay(0) result");
}
