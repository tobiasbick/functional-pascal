//! Conservative static reachability for the single-threaded V1 debugger gate.

use std::collections::VecDeque;

use fpas_bytecode::{Constant, FunctionId, Opcode, VerifiedExecutable};

pub(super) fn first_reachable_spawner(
    executable: &VerifiedExecutable,
) -> Option<fpas_bytecode::FunctionId> {
    let image = executable.executable();
    let mut reachable = vec![false; image.functions.len()];
    let mut queue = VecDeque::from([image.entry]);
    while let Some(function_id) = queue.pop_front() {
        let index = usize::from(function_id.get());
        if reachable.get(index).copied().unwrap_or(true) {
            continue;
        }
        reachable[index] = true;
        let function = &image.functions[index];
        if function.flags.uses_spawn_tasks {
            return Some(function_id);
        }
        let start = function.code.start.get() as usize;
        let end = function.code.end.get() as usize;
        for instruction in &image.code[start..end] {
            let Ok(opcode) = instruction.opcode() else {
                continue;
            };
            let target = match opcode {
                Opcode::CallDirect | Opcode::MakeClosure => instruction
                    .abc_operands()
                    .ok()
                    .map(|operands| FunctionId::new(operands.b)),
                Opcode::LoadConstant => instruction
                    .abx_operands()
                    .ok()
                    .and_then(|operands| image.constants.get(operands.bx as usize))
                    .and_then(|constant| match constant {
                        Constant::Function { function, .. } => Some(*function),
                        _ => None,
                    }),
                _ => None,
            };
            if let Some(target) = target {
                queue.push_back(target);
            }
        }
    }
    None
}
