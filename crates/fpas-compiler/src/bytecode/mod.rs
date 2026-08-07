//! Typed IR to verified register-bytecode construction for the development VM.

mod allocation;
mod blocks;
mod metadata;
mod selection;

use fpas_bytecode::{
    CodeRange, Executable, FunctionFlags, FunctionId, FunctionInfo, Instruction,
    InstructionAddress, NO_REGISTER, Opcode, ReturnConvention,
};
use fpas_ir::{BlockId, Function, IrType, Program, SourceSpan, Terminator};

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
    if program.entry != fpas_ir::FunctionId::new(0) || program.functions.is_empty() {
        return Err(compile_error(
            "register root function must use dense function identifier zero",
        ));
    }
    let (mut metadata, _) = MetadataBuilder::new(&program.functions[0].name)?;
    let mut code = Vec::new();
    let mut functions = Vec::with_capacity(program.functions.len());
    for (index, function) in program.functions.iter().enumerate() {
        if usize::try_from(function.id.get()).ok() != Some(index) {
            return Err(compile_error(
                "register function identifiers must be dense and ordered",
            ));
        }
        functions.push(compile_function(
            program,
            function,
            &mut code,
            &mut metadata,
        )?);
    }
    let (constants, strings, source_map) = metadata.finish();
    let executable = Executable {
        code,
        functions,
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

fn compile_function(
    program: &Program,
    function: &Function,
    code: &mut Vec<Instruction>,
    metadata: &mut MetadataBuilder,
) -> Result<FunctionInfo, CompileError> {
    let allocation = Allocation::build(function)?;
    let layout = BlockLayout::build_at(program, function, code.len())?;
    let name = metadata.function_name(&function.name)?;
    metadata.begin_function();
    let code_start = InstructionAddress::try_from_index(code.len())
        .map_err(|error| compile_error(&error.to_string()))?;
    let selector = Selector::new(program, function, &allocation);
    for (index, block) in function.blocks.iter().enumerate() {
        let mut source = None;
        for instruction in &block.instructions {
            source = instruction.source.or(source);
            for selected in selector.select(instruction, metadata)? {
                emit(code, metadata, instruction.source, selected)?;
            }
        }
        let terminator = block
            .terminators
            .first()
            .ok_or_else(|| compile_error("IR block has no terminator"))?;
        emit_terminator(
            code,
            metadata,
            &allocation,
            &layout,
            terminator,
            function.blocks.get(index + 1).map(|next| next.id),
            source,
        )?;
    }
    let code_end = InstructionAddress::try_from_index(code.len())
        .map_err(|error| compile_error(&error.to_string()))?;
    let arity = u8::try_from(function.parameters.len())
        .map_err(|_| compile_error("function arity exceeds u8"))?;
    let capture_count = u16::try_from(function.captures.len())
        .map_err(|_| compile_error("function capture count exceeds u16"))?;
    let return_convention = if matches!(
        program
            .ty(function.signature.result)
            .map(|definition| &definition.kind),
        Some(IrType::Unit)
    ) {
        ReturnConvention::Unit
    } else {
        ReturnConvention::Value
    };
    Ok(FunctionInfo {
        name,
        code: CodeRange::new(code_start, code_end),
        arity,
        capture_count,
        register_count: allocation.register_count,
        return_convention,
        flags: FunctionFlags::default(),
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
        Terminator::Return(Some(value)) => emit(
            code,
            metadata,
            source,
            abc(Opcode::Return, allocation.value(*value)?.get(), 0, 0)?,
        ),
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
