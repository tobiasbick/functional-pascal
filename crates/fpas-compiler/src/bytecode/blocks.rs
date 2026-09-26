//! Deterministic block layout and branch-width calculation.

use std::collections::BTreeMap;

use fpas_ir::{BlockId, Function, Terminator};

use crate::CompileError;
use crate::error::internal_compiler_error;

pub(super) struct BlockLayout {
    starts: BTreeMap<BlockId, u32>,
}

impl BlockLayout {
    /// Place blocks in order from their selected instruction and terminator word counts.
    pub fn from_widths(
        function: &Function,
        code_offset: usize,
        widths: &[usize],
    ) -> Result<Self, CompileError> {
        let mut starts = BTreeMap::new();
        let mut address = code_offset;
        for (block, width) in function.blocks.iter().zip(widths) {
            starts.insert(
                block.id,
                u32::try_from(address).map_err(|_| address_error())?,
            );
            address = address.checked_add(*width).ok_or_else(address_error)?;
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
        Terminator::ForLoop { .. } => 3,
        // Falling through to the next block needs no jump.
        Terminator::Jump(target) if next == Some(target.block) => 0,
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
