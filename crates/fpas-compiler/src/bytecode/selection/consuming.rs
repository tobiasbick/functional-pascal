//! Transfer a temporary instead of cloning it at its final read.

use fpas_bytecode::{Instruction, Opcode};
use fpas_ir::BlockId;

use super::{Selector, abc_aux};
use crate::CompileError;

impl Selector<'_> {
    /// Mark moves and wrapper operations whose source temporary dies at this instruction.
    ///
    /// Moving the last reference keeps copy-on-write values uniquely owned, so a following
    /// `IndexSet`, `UpdateRecord`, `ArrayPush`, or callee can update them in place.
    pub(super) fn mark_consuming_operands(
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
                if let Ok(
                    opcode @ (Opcode::Move | Opcode::MakeOk | Opcode::MakeError | Opcode::MakeSome),
                ) = word.opcode()
                    && consumable.contains(&operands.b)
                {
                    abc_aux(opcode, operands.a, operands.b, operands.c, 1)
                } else {
                    Ok(word)
                }
            })
            .collect()
    }
}
