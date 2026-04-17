//! Shared bytecode helpers for concurrency scheduling tests.

use crate::tests::helpers::{emit_constant, loc};
use fpas_bytecode::{Chunk, Op, Value};

/// Each pair is two instructions (`Constant` + `Pop`); every instruction counts toward the
/// timeslice budget. Keep total cost comfortably above the VM `TIMESLICE` constant (see
/// `crates/fpas-vm/src/vm/mod.rs`, currently 256) when the test must force rescheduling.
pub(crate) fn emit_instruction_waste(chunk: &mut Chunk, instruction_pairs: usize) {
    for _ in 0..instruction_pairs {
        emit_constant(chunk, Value::Integer(0));
        chunk.emit(Op::Pop, loc());
    }
}
