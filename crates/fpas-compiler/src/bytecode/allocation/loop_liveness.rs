//! Keep values that enter a loop alive for the whole loop.
//!
//! Linear-scan allocation frees a register after a value's last use in block order. A value
//! defined before a loop and read inside it must survive the back edge, so its last use extends
//! to the end of the latest block that jumps back to the loop header.

use std::collections::BTreeMap;

use fpas_ir::{Function, ValueId};

use super::{operation_values, terminator_values};

/// Extend `last_uses` for values that are defined before and read inside a loop.
pub(super) fn extend_across_back_edges(
    function: &Function,
    last_uses: &mut BTreeMap<ValueId, usize>,
) {
    let mut starts = Vec::with_capacity(function.blocks.len());
    let mut position = 0_usize;
    for block in &function.blocks {
        starts.push(position);
        position += block.instructions.len() + 1;
    }
    let indexes = function
        .blocks
        .iter()
        .enumerate()
        .map(|(index, block)| (block.id, index))
        .collect::<BTreeMap<_, _>>();
    // (header start, latch terminator position) for every back edge.
    let mut loops = Vec::new();
    for (index, block) in function.blocks.iter().enumerate() {
        for target in block
            .terminators
            .iter()
            .flat_map(|terminator| terminator.targets())
        {
            if let Some(&header) = indexes.get(&target.block)
                && header <= index
            {
                loops.push((starts[header], starts[index] + block.instructions.len()));
            }
        }
    }
    if loops.is_empty() {
        return;
    }
    let mut definitions = BTreeMap::new();
    for (index, block) in function.blocks.iter().enumerate() {
        for (offset, instruction) in block.instructions.iter().enumerate() {
            if let Some(result) = instruction.result {
                definitions.insert(result.id, starts[index] + offset);
            }
        }
    }
    let mut extend = |value: ValueId, position: usize| {
        let Some(&defined) = definitions.get(&value) else {
            return;
        };
        for &(header, latch) in &loops {
            if defined < header && (header..=latch).contains(&position) {
                let last_use = last_uses.entry(value).or_insert(latch);
                *last_use = (*last_use).max(latch);
            }
        }
    };
    for (index, block) in function.blocks.iter().enumerate() {
        for (offset, instruction) in block.instructions.iter().enumerate() {
            for value in operation_values(&instruction.operation) {
                extend(value, starts[index] + offset);
            }
        }
        for terminator in &block.terminators {
            for value in terminator_values(terminator) {
                extend(value, starts[index] + block.instructions.len());
            }
        }
    }
}
