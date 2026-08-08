//! Deterministic block layout and branch-width calculation.

use std::collections::BTreeMap;

use fpas_ir::{BlockId, Function, IrType, Operation, Program, Terminator};

use crate::CompileError;
use crate::error::internal_compiler_error;

use super::allocation::Allocation;

pub(super) struct BlockLayout {
    starts: BTreeMap<BlockId, u32>,
}

impl BlockLayout {
    pub fn build_at(
        program: &Program,
        function: &Function,
        allocation: &Allocation,
        code_offset: usize,
    ) -> Result<Self, CompileError> {
        let mut starts = BTreeMap::new();
        let mut address = u32::try_from(code_offset).map_err(|_| address_error())?;
        for (index, block) in function.blocks.iter().enumerate() {
            starts.insert(block.id, address);
            let instructions =
                block
                    .instructions
                    .iter()
                    .try_fold(0_u32, |width, instruction| {
                        width
                            .checked_add(operation_width(program, allocation, instruction)?)
                            .ok_or_else(address_error)
                    })?;
            address = address
                .checked_add(instructions)
                .ok_or_else(address_error)?;
            let terminator = block.terminators.first().ok_or_else(address_error)?;
            let width = terminator_width(
                terminator,
                function.blocks.get(index + 1).map(|next| next.id),
            );
            address = address.checked_add(width).ok_or_else(address_error)?;
        }
        Ok(Self { starts })
    }

    pub fn start(&self, block: BlockId) -> Result<u32, CompileError> {
        self.starts.get(&block).copied().ok_or_else(address_error)
    }
}

fn operation_width(
    program: &Program,
    allocation: &Allocation,
    instruction: &fpas_ir::Instruction,
) -> Result<u32, CompileError> {
    let width = match &instruction.operation {
        Operation::ReadLocal(local)
            if allocation.value(instruction.result.ok_or_else(address_error)?.id)?
                == allocation.local(*local)? =>
        {
            0
        }
        Operation::WriteLocal { value, local }
            if allocation.value(*value)? == allocation.local(*local)? =>
        {
            0
        }
        Operation::CallDirect {
            function,
            arguments,
        } => {
            let target = program.function(*function).ok_or_else(address_error)?;
            let unit = matches!(
                program
                    .ty(target.signature.result)
                    .map(|definition| &definition.kind),
                Some(IrType::Unit)
            );
            call_argument_width(arguments).saturating_add(1 + usize::from(unit))
        }
        Operation::CallValue { arguments, .. } => call_argument_width(arguments).saturating_add(1),
        Operation::SpawnTask { arguments, .. } | Operation::SpawnDetachedTask { arguments, .. } => {
            arguments.len().saturating_add(1)
        }
        Operation::Intrinsic {
            intrinsic,
            arguments,
        } => {
            let signature = program.intrinsic(*intrinsic).ok_or_else(address_error)?;
            let unit = matches!(
                program
                    .ty(signature.result)
                    .map(|definition| &definition.kind),
                Some(IrType::Unit)
            );
            arguments.len().saturating_add(1 + usize::from(unit))
        }
        Operation::MakeClosure { captures, .. } => captures.len().saturating_add(1),
        Operation::MakeArray(values) => values.len().saturating_add(1),
        Operation::MakeDictionary(pairs) => pairs.len().saturating_mul(2).saturating_add(1),
        Operation::StoreGlobalIndexPath { indexes, .. } => indexes.len().saturating_add(2),
        Operation::MakeRecord { fields, .. } | Operation::MakeEnum { fields, .. } => {
            fields.len().saturating_add(1)
        }
        Operation::IndexSet { .. } => 2,
        Operation::ArrayPush { .. } => 2,
        Operation::UpdateRecord { fields, .. } => fields.len().saturating_mul(2).saturating_add(2),
        _ => 1,
    };
    u32::try_from(width).map_err(|_| address_error())
}

fn call_argument_width(arguments: &[fpas_ir::ValueId]) -> usize {
    if arguments.len() == 1 {
        0
    } else {
        arguments.len()
    }
}

pub(super) fn terminator_width(terminator: &Terminator, next: Option<BlockId>) -> u32 {
    match terminator {
        Terminator::Branch {
            then_target,
            else_target,
            ..
        } if next == Some(then_target.block) || next == Some(else_target.block) => 1,
        Terminator::Branch { .. } => 2,
        Terminator::Jump(_) | Terminator::Return(_) | Terminator::Panic(_) => 1,
    }
}

fn address_error() -> CompileError {
    internal_compiler_error(
        "Register bytecode instruction-address limit exceeded or block target is missing.",
        "Split the program into smaller functions or report this compiler invariant failure.",
        1,
        1,
    )
}
