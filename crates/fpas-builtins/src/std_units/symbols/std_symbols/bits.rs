//! `Std.Math.Bits` symbol names and registry group.

std_symbol!(STD_BITS_BIT_AND = "Std.Math.Bits.BitAnd");
std_symbol!(STD_BITS_BIT_OR = "Std.Math.Bits.BitOr");
std_symbol!(STD_BITS_BIT_XOR = "Std.Math.Bits.BitXor");
std_symbol!(STD_BITS_BIT_NOT = "Std.Math.Bits.BitNot");
std_symbol!(STD_BITS_SHIFT_LEFT = "Std.Math.Bits.ShiftLeft");
std_symbol!(STD_BITS_SHIFT_RIGHT = "Std.Math.Bits.ShiftRight");

/// Public symbols provided by the intrinsic `Std.Math.Bits` unit.
pub(in crate::std_units) const STD_BITS_SYMBOLS: &[&str] = &[
    STD_BITS_BIT_AND,
    STD_BITS_BIT_OR,
    STD_BITS_BIT_XOR,
    STD_BITS_BIT_NOT,
    STD_BITS_SHIFT_LEFT,
    STD_BITS_SHIFT_RIGHT,
];
