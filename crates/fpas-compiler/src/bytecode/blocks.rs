//! Deterministic block layout and branch-width calculation.

use std::collections::BTreeMap;

use fpas_ir::{BlockId, Function, Terminator};

use crate::CompileError;
use crate::error::internal_compiler_error;

pub(super) struct BlockLayout {
    starts: BTreeMap<BlockId, u32>,
}

impl BlockLayout {
    pub fn build(function: &Function) -> Result<Self, CompileError> {
        let mut starts = BTreeMap::new();
        let mut address = 0_u32;
        for (index, block) in function.blocks.iter().enumerate() {
            starts.insert(block.id, address);
            let instructions =
                u32::try_from(block.instructions.len()).map_err(|_| address_error())?;
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
