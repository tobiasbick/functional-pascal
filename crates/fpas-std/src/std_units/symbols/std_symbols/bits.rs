//! `Std.Bits` symbol names; see `docs/pascal/std/numeric/bits.md`.
std_symbol!(STD_BITS_BIT_AND = "Std.Bits.BitAnd");
std_symbol!(STD_BITS_BIT_OR = "Std.Bits.BitOr");
std_symbol!(STD_BITS_BIT_XOR = "Std.Bits.BitXor");
std_symbol!(STD_BITS_BIT_NOT = "Std.Bits.BitNot");
std_symbol!(STD_BITS_SHIFT_LEFT = "Std.Bits.ShiftLeft");
std_symbol!(STD_BITS_SHIFT_RIGHT = "Std.Bits.ShiftRight");
/// Public operations supplied by `Std.Bits`.
pub(in crate::std_units) const STD_BITS_SYMBOLS: &[&str] = &[
    STD_BITS_BIT_AND,
    STD_BITS_BIT_OR,
    STD_BITS_BIT_XOR,
    STD_BITS_BIT_NOT,
    STD_BITS_SHIFT_LEFT,
    STD_BITS_SHIFT_RIGHT,
];
