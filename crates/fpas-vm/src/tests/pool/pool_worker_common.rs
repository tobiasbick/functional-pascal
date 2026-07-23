//! Helpers for pool worker loop integration tests.

use crate::vm::{SharedState, TaskResultPoll};
use fpas_bytecode::{Chunk, Op, Value};
use std::thread;
use std::time::Duration;

use crate::tests::helpers::{emit_constant, loc};

/// Bytecode for a spawned task starting at ip `0`: leave one return value and `Return`.
pub(crate) fn chunk_task_returns_integer(n: i64) -> Chunk {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(n));
    chunk.emit(Op::Return, loc());
    chunk
}

pub(crate) fn wait_for_task_result(shared: &SharedState, id: u64, timeout: Duration) -> Value {
    let start = std::time::Instant::now();
    loop {
        match shared.poll_task_result(id) {
            TaskResultPoll::Available(v) => return v,
            TaskResultPoll::Pending => {}
            TaskResultPoll::Consumed => panic!("task {id} result consumed before read"),
            TaskResultPoll::Unknown => panic!("task {id} was not registered"),
        }
        assert!(
            start.elapsed() < timeout,
            "timed out waiting for task {id} result"
        );
        thread::sleep(Duration::from_millis(2));
    }
}
