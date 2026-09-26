//! Copy propagation for local reads: reuse the local's register instead of copying it.
//!
//! A `ReadLocal` result may share the local's register when every use sits later in the same
//! block and no instruction writes the local before the last use. The last use itself may write
//! the local when it is a single-word operation that reads its operands before writing
//! (`I := I + 1`). Aliased reads emit no code.

use std::collections::BTreeMap;

use fpas_ir::{BlockId, Function, Instruction, LocalId, Operation, ValueId};

use super::{operation_values, terminator_values};

/// One local read that may reuse its local's register.
struct Candidate {
    block: BlockId,
    read: usize,
    local: LocalId,
    // Last reading instruction index in `block`, or the instruction count for a terminator use.
    last_use: Option<usize>,
    escapes_block: bool,
}

/// Map every aliasable `ReadLocal` result to the local whose register it reuses.
pub(super) fn aliased_local_reads(
    function: &Function,
    coalesced_writes: &BTreeMap<ValueId, LocalId>,
) -> BTreeMap<ValueId, LocalId> {
    let mut candidates = BTreeMap::new();
    for block in &function.blocks {
        for (read, instruction) in block.instructions.iter().enumerate() {
            if let (Operation::ReadLocal(local), Some(result)) =
                (&instruction.operation, instruction.result)
            {
                candidates.insert(
                    result.id,
                    Candidate {
                        block: block.id,
                        read,
                        local: *local,
                        last_use: None,
                        escapes_block: false,
                    },
                );
            }
        }
    }
    if candidates.is_empty() {
        return BTreeMap::new();
    }
    for block in &function.blocks {
        for (index, instruction) in block.instructions.iter().enumerate() {
            for value in operation_values(&instruction.operation) {
                record_use(&mut candidates, value, block.id, index);
            }
        }
        for terminator in &block.terminators {
            for value in terminator_values(terminator) {
                record_use(&mut candidates, value, block.id, block.instructions.len());
            }
        }
    }

    let blocks = function
        .blocks
        .iter()
        .map(|block| (block.id, (block, local_writes(block, coalesced_writes))))
        .collect::<BTreeMap<_, _>>();
    let mut aliases = BTreeMap::new();
    for (value, candidate) in &candidates {
        if candidate.escapes_block {
            continue;
        }
        let Some(last_use) = candidate.last_use else {
            aliases.insert(*value, candidate.local);
            continue;
        };
        let Some((block, writes)) = blocks.get(&candidate.block) else {
            continue;
        };
        let stays_valid = writes
            .get(&candidate.local)
            .into_iter()
            .flatten()
            .filter(|write| **write > candidate.read && **write <= last_use)
            .all(|write| {
                *write == last_use
                    && block
                        .instructions
                        .get(*write)
                        .is_some_and(|last| reads_before_writing(last, *value, candidate.local))
            });
        if stays_valid {
            aliases.insert(*value, candidate.local);
        }
    }
    aliases
}

fn record_use(
    candidates: &mut BTreeMap<ValueId, Candidate>,
    value: ValueId,
    block: BlockId,
    index: usize,
) {
    if let Some(candidate) = candidates.get_mut(&value) {
        if candidate.block == block && index > candidate.read {
            candidate.last_use = Some(candidate.last_use.map_or(index, |last| last.max(index)));
        } else {
            candidate.escapes_block = true;
        }
    }
}

/// Instruction indexes that write each local in `block`, in ascending order.
fn local_writes(
    block: &fpas_ir::BasicBlock,
    coalesced_writes: &BTreeMap<ValueId, LocalId>,
) -> BTreeMap<LocalId, Vec<usize>> {
    let mut writes = BTreeMap::<LocalId, Vec<usize>>::new();
    for (index, instruction) in block.instructions.iter().enumerate() {
        if let Some(local) = written_local(instruction, coalesced_writes) {
            writes.entry(local).or_default().push(index);
        }
    }
    writes
}

fn written_local(
    instruction: &Instruction,
    coalesced_writes: &BTreeMap<ValueId, LocalId>,
) -> Option<LocalId> {
    match &instruction.operation {
        Operation::WriteLocal { local, .. }
        | Operation::StoreLocalIndex { local, .. }
        | Operation::ArrayPush { local, .. }
        | Operation::ArrayPop { local } => Some(*local),
        _ => instruction
            .result
            .and_then(|result| coalesced_writes.get(&result.id).copied()),
    }
}

/// Single-word operations that read every operand before writing their destination.
fn reads_before_writing(instruction: &Instruction, value: ValueId, local: LocalId) -> bool {
    match &instruction.operation {
        Operation::Binary { .. } | Operation::Unary { .. } => true,
        Operation::WriteLocal {
            value: written,
            local: target,
        } => *written == value && *target == local,
        _ => false,
    }
}
