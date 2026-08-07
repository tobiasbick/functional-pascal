//! Entry-point control-flow validation for legacy stack chunks.

use crate::{Chunk, ExecutableError, Op};

pub(super) fn validate_entry_control_flow(chunk: &Chunk) -> Result<(), ExecutableError> {
    if chunk.is_empty() {
        return Err(ExecutableError::MissingEntryExit);
    }

    let function_regions = discover_function_regions(chunk);
    let mut visited = vec![false; chunk.len()];
    let mut pending = vec![(0, 0)];
    let mut reached_exit = false;

    while let Some((instruction, predecessor)) = pending.pop() {
        if function_regions[instruction] {
            return Err(ExecutableError::EntryFunctionRegion {
                instruction: predecessor,
                target: instruction,
            });
        }
        if std::mem::replace(&mut visited[instruction], true) {
            continue;
        }

        let op = chunk.code()[instruction];
        match op {
            Op::Halt | Op::Return => reached_exit = true,
            Op::Jump(target) => pending.push((target as usize, instruction)),
            Op::JumpIfFalse(target)
            | Op::JumpIfTrue(target)
            | Op::JumpIfLocalGt(_, _, target)
            | Op::JumpIfLocalLt(_, _, target) => {
                pending.push((target as usize, instruction));
                push_fallthrough(chunk, &mut pending, instruction)?;
            }
            _ => push_fallthrough(chunk, &mut pending, instruction)?,
        }
    }

    if reached_exit {
        Ok(())
    } else {
        Err(ExecutableError::MissingEntryExit)
    }
}

fn discover_function_regions(chunk: &Chunk) -> Vec<bool> {
    let mut regions = vec![false; chunk.len()];
    for &(entry, _) in chunk.functions().values() {
        if entry == 0 {
            continue;
        }
        let mut pending = vec![entry];
        while let Some(instruction) = pending.pop() {
            if instruction >= chunk.len() || std::mem::replace(&mut regions[instruction], true) {
                continue;
            }
            push_instruction_successors(chunk.code()[instruction], instruction, &mut pending);
        }
    }
    regions
}

fn push_instruction_successors(op: Op, instruction: usize, pending: &mut Vec<usize>) {
    match op {
        Op::Halt | Op::Return => {}
        Op::Jump(target) => pending.push(target as usize),
        Op::JumpIfFalse(target)
        | Op::JumpIfTrue(target)
        | Op::JumpIfLocalGt(_, _, target)
        | Op::JumpIfLocalLt(_, _, target) => {
            pending.push(target as usize);
            pending.push(instruction + 1);
        }
        _ => pending.push(instruction + 1),
    }
}

fn push_fallthrough(
    chunk: &Chunk,
    pending: &mut Vec<(usize, usize)>,
    instruction: usize,
) -> Result<(), ExecutableError> {
    let next = instruction + 1;
    if next >= chunk.len() {
        return Err(ExecutableError::EntryFallthrough { instruction });
    }
    pending.push((next, instruction));
    Ok(())
}
