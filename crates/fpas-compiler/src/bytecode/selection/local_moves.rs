//! Selection of local-register reads and writes.

use fpas_bytecode::Opcode;
use fpas_ir::{LocalId, ValueId};

use super::{Selector, abc};
use crate::CompileError;

impl Selector<'_> {
    pub(super) fn select_read_local(
        &self,
        local: LocalId,
        result: Option<ValueId>,
    ) -> Result<Vec<fpas_bytecode::Instruction>, CompileError> {
        self.select_move(
            self.result_register(result)?,
            self.allocation.local(local)?.get(),
        )
    }

    pub(super) fn select_write_local(
        &self,
        value: ValueId,
        local: LocalId,
    ) -> Result<Vec<fpas_bytecode::Instruction>, CompileError> {
        self.select_move(
            self.allocation.local(local)?.get(),
            self.allocation.value(value)?.get(),
        )
    }

    fn select_move(
        &self,
        destination: u16,
        source: u16,
    ) -> Result<Vec<fpas_bytecode::Instruction>, CompileError> {
        if destination == source {
            Ok(Vec::new())
        } else {
            Ok(vec![abc(Opcode::Move, destination, source, 0)?])
        }
    }
}
