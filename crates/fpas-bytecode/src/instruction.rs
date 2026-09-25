//! Packed register-bytecode instructions and safe form codecs.

mod error;
mod opcode;
mod operands;

pub use error::InstructionError;
pub use opcode::{InstructionForm, Opcode};
pub use operands::{AbcOperands, AbxOperands};

const OPCODE_BITS: u32 = 8;
const AX_MAX: u64 = (1_u64 << 48) - 1;

/// Exactly eight bytes of packed register bytecode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Instruction(u64);

impl Instruction {
    /// Construct an ABC-form instruction after checking the opcode declaration.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError::FormMismatch`] for a non-ABC opcode.
    pub const fn abc(
        opcode: Opcode,
        a: u16,
        b: u16,
        c: u16,
        auxiliary: u8,
    ) -> Result<Self, InstructionError> {
        if !matches!(opcode.form(), InstructionForm::Abc) {
            return Err(InstructionError::FormMismatch {
                opcode,
                expected: opcode.form(),
                actual: InstructionForm::Abc,
            });
        }
        let word = opcode as u64
            | ((a as u64) << 8)
            | ((b as u64) << 24)
            | ((c as u64) << 40)
            | ((auxiliary as u64) << 56);
        Ok(Self(word))
    }

    /// Construct an ABx-form instruction after checking the opcode declaration.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError::FormMismatch`] for a non-ABx opcode.
    pub const fn abx(opcode: Opcode, a: u16, bx: u32) -> Result<Self, InstructionError> {
        if !matches!(opcode.form(), InstructionForm::Abx) {
            return Err(InstructionError::FormMismatch {
                opcode,
                expected: opcode.form(),
                actual: InstructionForm::Abx,
            });
        }
        Ok(Self(
            opcode as u64 | ((a as u64) << 8) | ((bx as u64) << 24),
        ))
    }

    /// Construct an Ax-form instruction after checking its opcode and 48-bit range.
    ///
    /// # Errors
    ///
    /// Returns a form or payload error when the value cannot use the Ax encoding.
    pub const fn ax(opcode: Opcode, ax: u64) -> Result<Self, InstructionError> {
        if !matches!(opcode.form(), InstructionForm::Ax) {
            return Err(InstructionError::FormMismatch {
                opcode,
                expected: opcode.form(),
                actual: InstructionForm::Ax,
            });
        }
        if ax > AX_MAX {
            return Err(InstructionError::PayloadOutOfRange {
                form: InstructionForm::Ax,
                actual: ax,
                maximum: AX_MAX,
            });
        }
        Ok(Self(opcode as u64 | (ax << OPCODE_BITS)))
    }

    /// Construct an untrusted instruction candidate from its logical packed word.
    #[must_use]
    pub const fn from_word(word: u64) -> Self {
        Self(word)
    }

    /// Return the logical packed word for explicit little-endian encoding.
    #[must_use]
    pub const fn word(self) -> u64 {
        self.0
    }

    /// Decode the opcode, rejecting unknown discriminants.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError::UnknownOpcode`] when the low byte is unassigned.
    #[inline(always)]
    pub fn opcode(self) -> Result<Opcode, InstructionError> {
        let encoded = self.0.to_le_bytes()[0];
        Opcode::try_from(encoded).map_err(|_| InstructionError::UnknownOpcode(encoded))
    }

    /// Decode ABC operands after confirming the opcode form.
    ///
    /// # Errors
    ///
    /// Returns an opcode or form error for malformed input.
    #[inline(always)]
    pub fn abc_operands(self) -> Result<AbcOperands, InstructionError> {
        let opcode = self.opcode()?;
        ensure_form(opcode, InstructionForm::Abc)?;
        let bytes = self.0.to_le_bytes();
        Ok(AbcOperands {
            a: u16::from_le_bytes([bytes[1], bytes[2]]),
            b: u16::from_le_bytes([bytes[3], bytes[4]]),
            c: u16::from_le_bytes([bytes[5], bytes[6]]),
            auxiliary: bytes[7],
        })
    }

    /// Decode the raw ABC payload without checking the opcode or declared form.
    ///
    /// This is intended for consumers that already hold a verified executable.
    #[must_use]
    #[inline(always)]
    pub fn abc_payload(self) -> AbcOperands {
        let bytes = self.0.to_le_bytes();
        AbcOperands {
            a: u16::from_le_bytes([bytes[1], bytes[2]]),
            b: u16::from_le_bytes([bytes[3], bytes[4]]),
            c: u16::from_le_bytes([bytes[5], bytes[6]]),
            auxiliary: bytes[7],
        }
    }

    /// Decode ABx operands after confirming the opcode form.
    ///
    /// # Errors
    ///
    /// Returns an opcode or form error for malformed input.
    #[inline(always)]
    pub fn abx_operands(self) -> Result<AbxOperands, InstructionError> {
        let opcode = self.opcode()?;
        ensure_form(opcode, InstructionForm::Abx)?;
        let bytes = self.0.to_le_bytes();
        Ok(AbxOperands {
            a: u16::from_le_bytes([bytes[1], bytes[2]]),
            bx: u32::from_le_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]),
        })
    }

    /// Decode the raw ABx payload without checking the opcode or declared form.
    ///
    /// This is intended for consumers that already hold a verified executable.
    #[must_use]
    #[inline(always)]
    pub fn abx_payload(self) -> AbxOperands {
        let bytes = self.0.to_le_bytes();
        AbxOperands {
            a: u16::from_le_bytes([bytes[1], bytes[2]]),
            bx: u32::from_le_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]),
        }
    }

    /// Decode an Ax payload after confirming the opcode form.
    ///
    /// # Errors
    ///
    /// Returns an opcode or form error for malformed input.
    pub fn ax_operand(self) -> Result<u64, InstructionError> {
        let opcode = self.opcode()?;
        ensure_form(opcode, InstructionForm::Ax)?;
        Ok(self.0 >> OPCODE_BITS)
    }
}

#[inline(always)]
fn ensure_form(opcode: Opcode, actual: InstructionForm) -> Result<(), InstructionError> {
    let expected = opcode.form();
    if expected == actual {
        Ok(())
    } else {
        Err(InstructionError::FormMismatch {
            opcode,
            expected,
            actual,
        })
    }
}
