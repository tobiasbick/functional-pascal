//! Moves that transfer a temporary instead of cloning it at its final read.

use fpas_bytecode::{Instruction, Opcode};
use fpas_ir::BlockId;

use super::{Selector, abc_aux};
use crate::CompileError;

impl Selector<'_> {
    /// Set auxiliary 1 on emitted `Move`s whose source temporary dies at this instruction.
    ///
    /// Moving the last reference keeps copy-on-write values uniquely owned, so a following
    /// `IndexSet`, `UpdateRecord`, `ArrayPush`, or callee can update them in place.
    pub(super) fn mark_consuming_moves(
        &self,
        words: Vec<Instruction>,
        block: BlockId,
        index: usize,
        instruction: &fpas_ir::Instruction,
    ) -> Result<Vec<Instruction>, CompileError> {
        let consumable = self
            .allocation
            .consumable_registers(block, index, instruction);
        if consumable.is_empty() {
            return Ok(words);
        }
        words
            .into_iter()
            .map(|word| {
                let operands = word.abc_payload();
                if word.opcode() == Ok(Opcode::Move) && consumable.contains(&operands.b) {
                    abc_aux(Opcode::Move, operands.a, operands.b, operands.c, 1)
                } else {
                    Ok(word)
                }
            })
            .collect()
    }
}
