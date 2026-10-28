//! Packed instruction codec failures.

use std::fmt;

use super::{InstructionForm, Opcode};

/// A packed instruction cannot be decoded according to its opcode contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionError {
    /// The opcode byte has no assigned semantic operation.
    UnknownOpcode(u8),
    /// A constructor or accessor used the wrong payload form.
    FormMismatch {
        /// Opcode whose declaration was violated.
        opcode: Opcode,
        /// Form declared by the opcode.
        expected: InstructionForm,
        /// Form requested by the caller.
        actual: InstructionForm,
    },
    /// A logical payload exceeds its assigned bit width.
    PayloadOutOfRange {
        /// Payload form being encoded.
        form: InstructionForm,
        /// Requested logical value.
        actual: u64,
        /// Largest encodable logical value.
        maximum: u64,
    },
}

impl fmt::Display for InstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownOpcode(opcode) => write!(formatter, "unknown register opcode {opcode}"),
            Self::FormMismatch {
                opcode,
                expected,
                actual,
            } => write!(
                formatter,
                "opcode {opcode:?} uses {expected:?} operands, not {actual:?} operands"
            ),
            Self::PayloadOutOfRange {
                form,
                actual,
                maximum,
            } => write!(
                formatter,
                "{form:?} payload {actual} exceeds the maximum {maximum}"
            ),
        }
    }
}

impl std::error::Error for InstructionError {}
