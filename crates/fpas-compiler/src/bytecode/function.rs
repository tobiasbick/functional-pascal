//! Per-function register bytecode emission.
//!
//! Instructions are selected first; block addresses then follow from the actual word counts, so
//! selection may emit fewer words (aliased reads, dropped dead values) without a separate width
//! table.

use fpas_bytecode::{
    CodeRange, FunctionFlags, FunctionInfo, Instruction, InstructionAddress, NO_REGISTER, Opcode,
    ReturnConvention,
};
use fpas_ir::{BlockId, Function, IrType, Operation, Program, SourceSpan, Terminator};

use crate::CompileError;

use super::allocation::Allocation;
use super::blocks::{BlockLayout, terminator_width};
use super::compile_error;
use super::debug::compile_debug_info;
use super::metadata::MetadataBuilder;
use super::selection::{Selector, abc, abx};
use super::superinstructions::{convert_tail_call, fuse_integer_branch};

/// Code address associated with one IR instruction.
///
/// A local read aliased to its local's register emits no code; its value is consumed by a later
/// word, so its point moves to the next emitted word (possibly the terminator, as in
/// `return Local`). Other instructions without code, such as writes coalesced into the producing
/// instruction, already took effect earlier and get no point.
#[derive(Debug, Clone, Copy)]
pub(super) struct InstructionPoint {
    pub block: BlockId,
    pub instruction: usize,
    pub address: InstructionAddress,
    /// Whether the instruction itself emitted the word at `address`.
    pub emitted: bool,
}

struct SelectedInstruction {
    index: usize,
    source: Option<SourceSpan>,
    words: Vec<Instruction>,
    aliased_read: bool,
}

pub(super) fn compile_function(
    program: &Program,
    function: &Function,
    code: &mut Vec<Instruction>,
    metadata: &mut MetadataBuilder,
) -> Result<(FunctionInfo, Vec<InstructionPoint>), CompileError> {
    let allocation = Allocation::build(function)?;
    let name = metadata.function_name(&function.name)?;
    metadata.begin_function();
    let code_start = address_at(code.len())?;
    let selector = Selector::new(program, function, &allocation);
    let mut selected_blocks = Vec::with_capacity(function.blocks.len());
    let mut block_widths = Vec::with_capacity(function.blocks.len());
    for (index, block) in function.blocks.iter().enumerate() {
        let mut selected = Vec::with_capacity(block.instructions.len());
        let mut width = 0_usize;
        for (instruction_index, instruction) in block.instructions.iter().enumerate() {
            let words = selector.select(block, instruction_index, metadata)?;
            width += words.len();
            selected.push(SelectedInstruction {
                index: instruction_index,
                source: instruction.source,
                words,
                aliased_read: matches!(instruction.operation, Operation::ReadLocal(_)),
            });
        }
        let terminator = block
            .terminators
            .first()
            .ok_or_else(|| compile_error("IR block has no terminator"))?;
        if let (Some(last), Some(instruction)) = (selected.last_mut(), block.instructions.last()) {
            let before = last.words.len();
            convert_tail_call(
                program,
                function,
                &allocation,
                instruction,
                &mut last.words,
                terminator,
            )?;
            width -= before - last.words.len();
        }
        let next = function.blocks.get(index + 1).map(|next| next.id);
        width += terminator_width(terminator, next) as usize;
        selected_blocks.push(selected);
        block_widths.push(width);
    }
    let layout = BlockLayout::from_widths(function, code.len(), &block_widths)?;
    let mut points = Vec::new();
    let mut forwarded = Vec::new();
    for ((index, block), selected) in function.blocks.iter().enumerate().zip(selected_blocks) {
        let block_start = code.len();
        let mut source = None;
        for instruction in selected {
            source = instruction.source.or(source);
            if instruction.words.is_empty() {
                if instruction.aliased_read {
                    forwarded.push((block.id, instruction.index));
                }
                continue;
            }
            let address = address_at(code.len())?;
            forward_points(&mut points, &mut forwarded, address);
            points.push(InstructionPoint {
                block: block.id,
                instruction: instruction.index,
                address,
                emitted: true,
            });
            for word in instruction.words {
                emit(code, metadata, instruction.source, word)?;
            }
        }
        let terminator = block
            .terminators
            .first()
            .ok_or_else(|| compile_error("IR block has no terminator"))?;
        let terminator_start = code.len();
        forward_points(&mut points, &mut forwarded, address_at(terminator_start)?);
        emit_terminator(
            code,
            metadata,
            &allocation,
            &layout,
            terminator,
            function.blocks.get(index + 1).map(|next| next.id),
            source,
        )?;
        if matches!(terminator, Terminator::Branch { .. }) && terminator_start > block_start {
            fuse_integer_branch(code, terminator_start)?;
        }
    }
    let code_end = address_at(code.len())?;
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
    let debug = compile_debug_info(program, function, &allocation, &points, metadata)?;
    Ok((
        FunctionInfo {
            name,
            code: CodeRange::new(code_start, code_end),
            arity,
            capture_count,
            register_count: allocation.register_count,
            return_convention,
            flags: FunctionFlags {
                uses_spawn_tasks: function.can_spawn_tasks,
            },
            debug,
        },
        points,
    ))
}

fn forward_points(
    points: &mut Vec<InstructionPoint>,
    forwarded: &mut Vec<(BlockId, usize)>,
    address: InstructionAddress,
) {
    points.extend(
        forwarded
            .drain(..)
            .map(|(block, instruction)| InstructionPoint {
                block,
                instruction,
                address,
                emitted: false,
            }),
    );
}

fn address_at(index: usize) -> Result<InstructionAddress, CompileError> {
    InstructionAddress::try_from_index(index).map_err(|error| compile_error(&error.to_string()))
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
        Terminator::Jump(target) if next == Some(target.block) => Ok(()),
        Terminator::Jump(target) => emit(
            code,
            metadata,
            source,
            abx(Opcode::Jump, 0, layout.start(target.block)?)?,
        ),
        Terminator::ForLoop {
            counter,
            bound,
            descending,
            body_target,
            after_target,
        } => {
            emit(
                code,
                metadata,
                source,
                Instruction::abc(
                    Opcode::ForLoop,
                    allocation.local(*counter)?.get(),
                    allocation.local(*bound)?.get(),
                    0,
                    u8::from(*descending),
                )
                .map_err(|error| compile_error(&error.to_string()))?,
            )?;
            emit(
                code,
                metadata,
                source,
                abx(Opcode::Jump, 0, layout.start(body_target.block)?)?,
            )?;
            emit(
                code,
                metadata,
                source,
                abx(Opcode::Jump, 0, layout.start(after_target.block)?)?,
            )
        }
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
