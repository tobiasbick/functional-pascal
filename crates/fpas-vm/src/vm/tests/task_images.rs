//! Assemble explicit scope ownership around task-spawning hand-built test routines.

use fpas_bytecode::{CodeRange, Executable, Instruction, InstructionAddress, Opcode};

/// Add balanced scope instructions and relocate the fixture's code/debug addresses.
pub(in crate::vm) fn scope_owned(mut image: Executable) -> Executable {
    let original = std::mem::take(&mut image.code);
    assert!(
        !original.iter().any(|word| matches!(
            word.opcode(),
            Ok(Opcode::BranchIfEqualInteger
                | Opcode::BranchIfNotEqualInteger
                | Opcode::BranchIfLessInteger
                | Opcode::BranchIfGreaterInteger
                | Opcode::BranchIfLessEqualInteger
                | Opcode::BranchIfGreaterEqualInteger)
        )),
        "scope-owned fixture relocation does not support fused integer branches"
    );
    let mut enter = vec![false; original.len()];
    let mut exit = vec![false; original.len()];
    for function in &image.functions {
        let start = function.code.start.get() as usize;
        let end = function.code.end.get() as usize;
        let body = &original[start..end];
        if body
            .iter()
            .any(|word| word.opcode() == Ok(Opcode::SpawnTask))
            && !body
                .iter()
                .any(|word| word.opcode() == Ok(Opcode::EnterTaskScope))
        {
            enter[start] = true;
            for index in start..end {
                exit[index] = original[index].opcode() == Ok(Opcode::Return);
            }
        }
    }
    let mut boundaries = Vec::with_capacity(original.len() + 1);
    let mut instructions = Vec::with_capacity(original.len());
    for (index, word) in original.into_iter().enumerate() {
        boundaries.push(image.code.len() as u32);
        if enter[index] {
            image
                .code
                .push(Instruction::abc(Opcode::EnterTaskScope, 0, 0, 0, 0).unwrap());
        }
        if exit[index] {
            image
                .code
                .push(Instruction::abc(Opcode::ExitTaskScope, 0, 0, 0, 0).unwrap());
        }
        instructions.push(image.code.len() as u32);
        image.code.push(word);
    }
    boundaries.push(image.code.len() as u32);
    let boundary =
        |address: InstructionAddress| InstructionAddress::new(boundaries[address.get() as usize]);
    let instruction =
        |address: InstructionAddress| InstructionAddress::new(instructions[address.get() as usize]);
    for &index in &instructions {
        let word = &mut image.code[index as usize];
        if matches!(
            word.opcode(),
            Ok(Opcode::Jump | Opcode::BranchIfFalse | Opcode::BranchIfTrue)
        ) {
            let operands = word.abx_operands().unwrap();
            let target = operands.bx as usize;
            *word = Instruction::abx(
                word.opcode().unwrap(),
                operands.a,
                if exit[target] {
                    boundaries[target]
                } else {
                    instructions[target]
                },
            )
            .unwrap();
        }
    }
    for function in &mut image.functions {
        function.code = CodeRange::new(boundary(function.code.start), boundary(function.code.end));
        for point in &mut function.debug.sequence_points {
            point.instruction = boundary(point.instruction);
        }
        for binding in &mut function.debug.bindings {
            binding.initializer = binding.initializer.map(instruction);
        }
    }
    for global in &mut image.globals {
        if let Some(initializer) = &mut global.initializer {
            initializer.instruction = instruction(initializer.instruction);
        }
    }
    for run in &mut image.source_map.runs {
        run.instruction_start = boundary(run.instruction_start);
    }
    image
}
