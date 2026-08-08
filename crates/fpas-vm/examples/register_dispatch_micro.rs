//! Reproducible local dispatch measurement for the inactive P3 register interpreter.

use std::error::Error;
use std::time::Instant;

use fpas_bytecode::{
    CodeRange, Constant, Executable, FunctionFlags, FunctionId, FunctionInfo, Instruction,
    InstructionAddress, Opcode, ReturnConvention, SourceId, SourceMap, SourceRun, StringId,
    StringTable,
};
use fpas_vm::Vm;

fn main() -> Result<(), Box<dyn Error>> {
    let iterations = match std::env::args().nth(1) {
        Some(value) => value.parse::<i64>()?,
        None => 10_000_000,
    };
    if iterations <= 0 {
        return Err("iteration count must be positive".into());
    }

    let mut vm = Vm::new(loop_executable(iterations)?);
    let started = Instant::now();
    let execution = vm
        .run()
        .map_err(|error| std::io::Error::other(format!("{}: {}", error.code, error.message)))?;
    let elapsed = started.elapsed();
    let throughput = execution.instruction_count as f64 / elapsed.as_secs_f64() / 1_000_000.0;
    println!(
        "iterations={iterations} instructions={} elapsed_ms={:.3} million_instructions_per_second={throughput:.3}",
        execution.instruction_count,
        elapsed.as_secs_f64() * 1_000.0,
    );
    Ok(())
}

fn loop_executable(iterations: i64) -> Result<fpas_bytecode::VerifiedExecutable, Box<dyn Error>> {
    let code = vec![
        Instruction::abx(Opcode::LoadConstant, 0, 0)?,
        Instruction::abx(Opcode::LoadConstant, 1, 1)?,
        Instruction::abc(Opcode::SubtractInteger, 0, 0, 1, 0)?,
        Instruction::abx(Opcode::BranchIfTrue, 0, 2)?,
        Instruction::abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0)?,
    ];
    let executable = Executable {
        code,
        functions: vec![FunctionInfo {
            name: StringId::new(0),
            code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(5)),
            arity: 0,
            capture_count: 0,
            register_count: 2,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
        }],
        constants: vec![Constant::Integer(iterations), Constant::Integer(1)],
        strings: StringTable::new(vec!["dispatch_micro".to_owned(), "<micro>".to_owned()]),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        source_map: SourceMap {
            sources: vec![StringId::new(1)],
            runs: vec![SourceRun {
                instruction_start: InstructionAddress::new(0),
                source: SourceId::new(0),
                line: 1,
                column: 1,
            }],
        },
        entry: FunctionId::new(0),
    };
    Ok(executable.verify()?)
}
