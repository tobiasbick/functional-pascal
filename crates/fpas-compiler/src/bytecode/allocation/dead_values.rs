//! Constants that no emitted instruction reads.
//!
//! A constant is dead when nothing reads it from a register, for example after constant folding
//! consumed its operands, or when every use encodes it as an immediate operand
//! (`AddIntegerImm`, `DivideIntegerImm`). Dead constants get no register and emit no code.

use std::collections::{BTreeMap, BTreeSet};

use fpas_ir::{BasicBlock, Function, Operation, ValueId};

use super::super::superinstructions::integer_immediate;

/// The operand of instruction `index` that selection encodes as an immediate, if any.
pub(super) fn immediate_operand(block: &BasicBlock, index: usize) -> Option<ValueId> {
    let Operation::Binary {
        operation,
        left,
        right,
    } = &block.instructions.get(index)?.operation
    else {
        return None;
    };
    let previous = index
        .checked_sub(1)
        .and_then(|index| block.instructions.get(index));
    integer_immediate(previous, *operation, *left, *right).map(|_| *right)
}

/// Results of `Const` instructions that never appear in `register_uses`.
pub(super) fn dead_constants(
    function: &Function,
    register_uses: &BTreeMap<ValueId, usize>,
) -> BTreeSet<ValueId> {
    function
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .filter(|instruction| matches!(instruction.operation, Operation::Const(_)))
        .filter_map(|instruction| instruction.result.map(|result| result.id))
        .filter(|value| !register_uses.contains_key(value))
        .collect()
}
