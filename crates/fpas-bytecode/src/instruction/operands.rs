//! Decoded packed instruction forms.

/// Decoded ABC-form operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbcOperands {
    /// First 16-bit operand.
    pub a: u16,
    /// Second 16-bit operand.
    pub b: u16,
    /// Third 16-bit operand.
    pub c: u16,
    /// Auxiliary 8-bit operand.
    pub auxiliary: u8,
}

/// Decoded ABx-form operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbxOperands {
    /// First 16-bit operand.
    pub a: u16,
    /// Second 32-bit operand.
    pub bx: u32,
}
