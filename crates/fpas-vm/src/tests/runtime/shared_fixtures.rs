//! Minimal chunks and [`TaskState`](crate::vm::TaskState) builders for `SharedState` tests.

use crate::tests::helpers::loc;
use crate::vm::TaskState;
use fpas_bytecode::{Chunk, Op};

pub(crate) fn minimal_halt_chunk() -> Chunk {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, loc());
    chunk
}

/// Build the smallest valid spawned-task body.
pub(crate) fn minimal_return_chunk() -> Chunk {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::Return, loc());
    chunk
}

pub(crate) fn dummy_task(id: u64, ip: usize) -> TaskState {
    TaskState {
        id,
        ip,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: false,
    }
}
