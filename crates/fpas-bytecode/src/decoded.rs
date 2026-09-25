//! Operand data prepared once for execution of verified bytecode.

use crate::{AbcOperands, AbxOperands, Instruction, InstructionError, Opcode};

/// Opcode and operands decoded from one validated instruction.
///
/// The layout mirrors the packed eight-byte encoding: ABC operands `b` and `c` occupy the low and
/// high halves of the ABx operand `bx`, so both forms are read without further checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedInstruction {
    bx: u32,
    a: u16,
    opcode: Opcode,
    auxiliary: u8,
}

impl DecodedInstruction {
    pub(crate) fn decode(instruction: Instruction) -> Result<Self, InstructionError> {
        let opcode = instruction.opcode()?;
        let abc = instruction.abc_payload();
        Ok(Self {
            bx: instruction.abx_payload().bx,
            a: abc.a,
            opcode,
            auxiliary: abc.auxiliary,
        })
    }

    /// Return the opcode checked when the executable was verified.
    #[must_use]
    #[inline(always)]
    pub const fn opcode(self) -> Opcode {
        self.opcode
    }

    /// Return pre-decoded ABC operands for an ABC-form opcode.
    #[must_use]
    #[inline(always)]
    pub const fn abc(self) -> AbcOperands {
        AbcOperands {
            a: self.a,
            b: self.bx as u16,
            c: (self.bx >> 16) as u16,
            auxiliary: self.auxiliary,
        }
    }

    /// Return pre-decoded ABx operands for an ABx-form opcode.
    #[must_use]
    #[inline(always)]
    pub const fn abx(self) -> AbxOperands {
        AbxOperands {
            a: self.a,
            bx: self.bx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_forms_preserve_the_packed_operands() {
        let abc = Instruction::abc(Opcode::AddInteger, 1, 2, 3, 4).expect("ABC instruction");
        let abx = Instruction::abx(Opcode::LoadConstant, 5, 0x1234_5678).expect("ABx instruction");
        let abc = DecodedInstruction::decode(abc).expect("decode ABC");
        let abx = DecodedInstruction::decode(abx).expect("decode ABx");
        assert_eq!(abc.opcode(), Opcode::AddInteger);
        assert_eq!(abc.abc().a, 1);
        assert_eq!(abc.abc().b, 2);
        assert_eq!(abc.abc().c, 3);
        assert_eq!(abc.abc().auxiliary, 4);
        assert_eq!(abx.opcode(), Opcode::LoadConstant);
        assert_eq!(abx.abx().a, 5);
        assert_eq!(abx.abx().bx, 0x1234_5678);
    }

    #[test]
    fn full_width_abc_operands_survive_the_shared_bx_slot() {
        let packed =
            Instruction::abc(Opcode::Move, 0xFFFE, 0xFFFF, 0x8001, 0xFF).expect("ABC instruction");
        let decoded = DecodedInstruction::decode(packed).expect("decode ABC");
        assert_eq!(decoded.abc(), packed.abc_payload());
    }

    #[test]
    fn decoded_instructions_keep_the_packed_size() {
        assert_eq!(std::mem::size_of::<DecodedInstruction>(), 8);
    }
}
