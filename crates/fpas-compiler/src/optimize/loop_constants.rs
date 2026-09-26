//! Loop-invariant code motion for constants.
//!
//! Constants inside a loop move to the end of the loop's single outside predecessor, so each is
//! loaded once per loop entry instead of once per iteration. A constant that selection encodes as
//! an immediate operand of the next instruction stays in place. Debugger locations of the
//! remaining instructions are remapped; moved constants keep no sequence point, because their
//! code now runs before the loop.

use std::collections::BTreeMap;

use fpas_ir::{BlockId, DebugInstructionLocation, Function, Instruction, Operation, Program};

use crate::bytecode::integer_immediate;

/// Original instruction index -> current index per changed block; `None` marks a moved constant.
type IndexMap = BTreeMap<BlockId, Vec<Option<usize>>>;

/// Hoist loop constants in every function of `program`.
pub(super) fn hoist_loop_constants(program: &mut Program) {
    for function in &mut program.functions {
        let map = hoist_function(function);
        if map.is_empty() {
            continue;
        }
        remap_function_debug(function, &map);
        for global in &mut program.globals {
            if let Some(initializer) = &mut global.initializer
                && initializer.function == function.id
                && let Some(location) = remap(initializer.location, &map)
            {
                initializer.location = location;
            }
        }
    }
}

fn hoist_function(function: &mut Function) -> IndexMap {
    let positions = function
        .blocks
        .iter()
        .enumerate()
        .map(|(index, block)| (block.id, index))
        .collect::<BTreeMap<_, _>>();
    // Loop header index -> last back-edge source index.
    let mut loops = BTreeMap::<usize, usize>::new();
    for (index, block) in function.blocks.iter().enumerate() {
        for target in block
            .terminators
            .iter()
            .flat_map(|terminator| terminator.targets())
        {
            if let Some(&header) = positions.get(&target.block)
                && header <= index
            {
                let latch = loops.entry(header).or_insert(index);
                *latch = (*latch).max(index);
            }
        }
    }
    let mut map = IndexMap::new();
    // Inner loops start later; hoisting them first lets an enclosing loop move the same
    // constants further out.
    for (&header, &latch) in loops.iter().rev() {
        let Some(preheader) = single_outside_predecessor(function, header) else {
            continue;
        };
        // Current instruction indexes to move, per block of the loop.
        let mut removals = (header..=latch)
            .map(|block_index| {
                let instructions = &function.blocks[block_index].instructions;
                (0..instructions.len())
                    .filter(|index| is_hoistable(instructions, *index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        keep_one_sequence_point(function, &map, header, &mut removals);
        let mut hoisted = Vec::new();
        for (block_index, removed) in (header..=latch).zip(removals) {
            if removed.is_empty() {
                continue;
            }
            let block = &mut function.blocks[block_index];
            let length = block.instructions.len();
            let indexes = map
                .entry(block.id)
                .or_insert_with(|| (0..length).map(Some).collect());
            for index in indexes.iter_mut() {
                *index = index.and_then(|current| {
                    removed
                        .binary_search(&current)
                        .is_err()
                        .then(|| current - removed.partition_point(|removed| *removed < current))
                });
            }
            let mut kept = Vec::with_capacity(length - removed.len());
            for (index, instruction) in block.instructions.drain(..).enumerate() {
                if removed.binary_search(&index).is_ok() {
                    hoisted.push(instruction);
                } else {
                    kept.push(instruction);
                }
            }
            block.instructions = kept;
        }
        // Appended after existing instructions, so their indexes stay unchanged.
        function.blocks[preheader].instructions.extend(hoisted);
    }
    map
}

/// Leave the first constant with a sequence point in place when moving every candidate would
/// leave the loop without any sequence point; the debugger pauses running code only there.
fn keep_one_sequence_point(
    function: &Function,
    map: &IndexMap,
    header: usize,
    removals: &mut [Vec<usize>],
) {
    let current_points = function.debug.sequence_points.iter().filter_map(|point| {
        let location = remap(
            DebugInstructionLocation {
                block: point.block,
                instruction: point.instruction,
            },
            map,
        )?;
        let offset = function.blocks[header..]
            .iter()
            .take(removals.len())
            .position(|block| block.id == location.block)?;
        Some((offset, location.instruction))
    });
    let mut first_removed_point = None;
    for (offset, instruction) in current_points {
        if removals[offset].binary_search(&instruction).is_err() {
            return;
        }
        first_removed_point.get_or_insert((offset, instruction));
    }
    if let Some((offset, instruction)) = first_removed_point
        && let Ok(position) = removals[offset].binary_search(&instruction)
    {
        removals[offset].remove(position);
    }
}

/// A constant that the next instruction neither encodes as an immediate operand nor stores
/// directly into a local (both already cost no separate load).
fn is_hoistable(instructions: &[Instruction], index: usize) -> bool {
    let instruction = &instructions[index];
    let (Operation::Const(_), Some(result)) = (&instruction.operation, instruction.result) else {
        return false;
    };
    !instructions
        .get(index + 1)
        .is_some_and(|next| match &next.operation {
            Operation::Binary {
                operation,
                left,
                right,
            } => {
                *right == result.id
                    && integer_immediate(Some(instruction), *operation, *left, *right).is_some()
            }
            Operation::WriteLocal { value, .. } => *value == result.id,
            _ => false,
        })
}

/// The only block before `header` that branches to it, when there is exactly one.
fn single_outside_predecessor(function: &Function, header: usize) -> Option<usize> {
    let header_id = function.blocks.get(header)?.id;
    let mut outside = function.blocks[..header]
        .iter()
        .enumerate()
        .filter(|(_, block)| {
            block
                .terminators
                .iter()
                .flat_map(|terminator| terminator.targets())
                .any(|target| target.block == header_id)
        })
        .map(|(index, _)| index);
    let first = outside.next()?;
    outside.next().is_none().then_some(first)
}

fn remap_function_debug(function: &mut Function, map: &IndexMap) {
    function.debug.sequence_points.retain_mut(|point| {
        let location = DebugInstructionLocation {
            block: point.block,
            instruction: point.instruction,
        };
        match remap(location, map) {
            Some(location) => {
                point.instruction = location.instruction;
                true
            }
            None => false,
        }
    });
    for binding in &mut function.debug.bindings {
        if let Some(initializer) = binding.initializer
            && let Some(location) = remap(initializer, map)
        {
            binding.initializer = Some(location);
        }
    }
}

/// Current location of an original instruction, or `None` when it was a moved constant.
fn remap(location: DebugInstructionLocation, map: &IndexMap) -> Option<DebugInstructionLocation> {
    let Some(indexes) = map.get(&location.block) else {
        return Some(location);
    };
    let instruction = (*indexes.get(location.instruction)?)?;
    Some(DebugInstructionLocation {
        block: location.block,
        instruction,
    })
}
