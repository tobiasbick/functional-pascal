use std::sync::Arc;

use fpas_bytecode::{
    CodeRange, Constant, Executable, FunctionFlags, FunctionId, FunctionInfo, Instruction,
    InstructionAddress, Opcode, ReturnConvention, SourceId, SourceMap, SourceRun, StringId,
    StringTable, Value, VerifiedExecutable,
};

use super::super::dispatch::DispatchStep;
use super::super::worker::Worker;
use crate::vm::VmError;

pub(super) fn abc(opcode: Opcode, a: u16, b: u16, c: u16) -> Instruction {
    Instruction::abc(opcode, a, b, c, 0).expect("test instruction must use ABC")
}

/// Builds an ABC test instruction with an explicit auxiliary operand.
pub(super) fn abc_aux(opcode: Opcode, a: u16, b: u16, c: u16, auxiliary: u8) -> Instruction {
    Instruction::abc(opcode, a, b, c, auxiliary).expect("test instruction must use ABC")
}

pub(super) fn abx(opcode: Opcode, a: u16, bx: u32) -> Instruction {
    Instruction::abx(opcode, a, bx).expect("test instruction must use ABx")
}

pub(super) fn verified(
    code: Vec<Instruction>,
    constants: Vec<Constant>,
    strings: Vec<&str>,
    register_count: u16,
) -> VerifiedExecutable {
    unverified(code, constants, strings, register_count)
        .verify()
        .expect("test executable must verify")
}

pub(super) fn unverified(
    code: Vec<Instruction>,
    constants: Vec<Constant>,
    strings: Vec<&str>,
    register_count: u16,
) -> Executable {
    let code_len = u32::try_from(code.len()).expect("test code length must fit u32");
    Executable {
        code,
        functions: vec![FunctionInfo {
            name: StringId::new(0),
            code: CodeRange::new(
                InstructionAddress::new(0),
                InstructionAddress::new(code_len),
            ),
            arity: 0,
            capture_count: 0,
            register_count,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
        }],
        constants,
        strings: StringTable::new(strings.into_iter().map(str::to_owned).collect()),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        source_map: SourceMap {
            sources: vec![StringId::new(1)],
            runs: vec![SourceRun {
                instruction_start: InstructionAddress::new(0),
                source: SourceId::new(0),
                line: 41,
                column: 7,
            }],
        },
        entry: FunctionId::new(0),
    }
}

pub(super) fn execute(executable: VerifiedExecutable) -> Result<(Value, Vec<Value>, u64), VmError> {
    let mut worker = Worker::new(Arc::new(executable))?;
    loop {
        match worker.dispatch_one()? {
            DispatchStep::Continue => {}
            DispatchStep::Suspend => panic!("test execution suspended without a scheduler"),
            DispatchStep::Return(value) => {
                return Ok((value, worker.registers, worker.instruction_count));
            }
        }
    }
}

pub(super) fn return_unit() -> Instruction {
    abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0)
}
