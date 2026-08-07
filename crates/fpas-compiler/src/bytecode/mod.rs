//! Typed IR to verified register-bytecode construction for the P3 subset.

mod allocation;
mod blocks;
mod metadata;
mod selection;

use fpas_bytecode::{
    CodeRange, Executable, FunctionFlags, FunctionId, FunctionInfo, Instruction,
    InstructionAddress, NO_REGISTER, Opcode, ReturnConvention,
};
use fpas_ir::{BlockId, Program, SourceSpan, Terminator};

use crate::CompileError;
use crate::error::internal_compiler_error;

use self::allocation::Allocation;
use self::blocks::BlockLayout;
use self::metadata::MetadataBuilder;
use self::selection::{Selector, abc, abx};

pub(super) fn compile_program(
    program: &Program,
) -> Result<fpas_bytecode::VerifiedExecutable, CompileError> {
    program
        .validate()
        .map_err(|error| compile_error(&error.to_string()))?;
    let [function] = program.functions.as_slice() else {
        return Err(compile_error(
            "P3 bytecode construction requires exactly one root function",
        ));
    };
    if program.entry != function.id || function.id != fpas_ir::FunctionId::new(0) {
        return Err(compile_error(
            "P3 root function must use dense function identifier zero",
        ));
    }

    let allocation = Allocation::build(function)?;
    let layout = BlockLayout::build(function)?;
    let (mut metadata, function_name) = MetadataBuilder::new(&function.name)?;
    let mut code = Vec::new();
    {
        let selector = Selector::new(program, function, &allocation);
        for (index, block) in function.blocks.iter().enumerate() {
            let mut source = None;
            for instruction in &block.instructions {
                source = instruction.source.or(source);
                let selected = selector.select(instruction, &mut metadata)?;
                emit(&mut code, &mut metadata, instruction.source, selected)?;
            }
            let terminator = block
                .terminators
                .first()
                .ok_or_else(|| compile_error("IR block has no terminator"))?;
            emit_terminator(
                &mut code,
                &mut metadata,
                &allocation,
                &layout,
                terminator,
                function.blocks.get(index + 1).map(|next| next.id),
                source,
            )?;
        }
    }
    let code_end = InstructionAddress::try_from_index(code.len())
        .map_err(|error| compile_error(&error.to_string()))?;
    let (constants, strings, source_map) = metadata.finish();
    let executable = Executable {
        code,
        functions: vec![FunctionInfo {
            name: function_name,
            code: CodeRange::new(InstructionAddress::new(0), code_end),
            arity: 0,
            capture_count: 0,
            register_count: allocation.register_count,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
        }],
        constants,
        strings,
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        source_map,
        entry: FunctionId::new(0),
    };
    executable.verify().map_err(|error| {
        compile_error(&format!(
            "generated executable failed verification: {error}"
        ))
    })
}

fn emit_terminator(
    code: &mut Vec<Instruction>,
    metadata: &mut MetadataBuilder,
    allocation: &Allocation,
    layout: &BlockLayout,
    terminator: &Terminator,
    next: Option<BlockId>,
    source: Option<SourceSpan>,
) -> Result<(), CompileError> {
    match terminator {
        Terminator::Branch {
            condition,
            then_target,
            else_target,
        } => {
            let condition = allocation.value(*condition)?.get();
            if next == Some(then_target.block) {
                emit(
                    code,
                    metadata,
                    source,
                    abx(
                        Opcode::BranchIfFalse,
                        condition,
                        layout.start(else_target.block)?,
                    )?,
                )
            } else if next == Some(else_target.block) {
                emit(
                    code,
                    metadata,
                    source,
                    abx(
                        Opcode::BranchIfTrue,
                        condition,
                        layout.start(then_target.block)?,
                    )?,
                )
            } else {
                emit(
                    code,
                    metadata,
                    source,
                    abx(
                        Opcode::BranchIfFalse,
                        condition,
                        layout.start(else_target.block)?,
                    )?,
                )?;
                emit(
                    code,
                    metadata,
                    source,
                    abx(Opcode::Jump, 0, layout.start(then_target.block)?)?,
                )
            }
        }
        Terminator::Jump(target) => emit(
            code,
            metadata,
            source,
            abx(Opcode::Jump, 0, layout.start(target.block)?)?,
        ),
        Terminator::Return(None) => emit(
            code,
            metadata,
            source,
            abc(Opcode::Return, NO_REGISTER, 0, 0)?,
        ),
        Terminator::Return(Some(_)) => Err(compile_error(
            "P3 root entry cannot return a value; function returns are implemented in P4",
        )),
        Terminator::Panic(value) => emit(
            code,
            metadata,
            source,
            abc(Opcode::Panic, allocation.value(*value)?.get(), 0, 0)?,
        ),
    }
}

fn emit(
    code: &mut Vec<Instruction>,
    metadata: &mut MetadataBuilder,
    source: Option<SourceSpan>,
    instruction: Instruction,
) -> Result<(), CompileError> {
    metadata.record_source(code.len(), source)?;
    code.push(instruction);
    Ok(())
}

fn compile_error(message: &str) -> CompileError {
    internal_compiler_error(
        format!("Register bytecode construction failed: {message}."),
        "This is an internal compiler error. Re-run compilation and report the source program.",
        1,
        1,
    )
}
