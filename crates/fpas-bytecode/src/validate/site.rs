//! Position of the instruction whose operands are being validated.

use crate::{Executable, FunctionId, FunctionInfo, InstructionAddress, Opcode};

/// Executable, owning function, address, and opcode of one decoded instruction.
#[derive(Clone, Copy)]
pub(super) struct InstructionSite<'a> {
    pub(super) executable: &'a Executable,
    pub(super) function_id: FunctionId,
    pub(super) function: &'a FunctionInfo,
    pub(super) address: InstructionAddress,
    pub(super) opcode: Opcode,
}
