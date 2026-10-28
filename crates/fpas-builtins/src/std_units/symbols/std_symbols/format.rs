//! `Std.Text.Format` symbol names and registry group.

std_symbol!(STD_FORMAT_INT_TO_STR = std_format!("IntToStr"));
std_symbol!(STD_FORMAT_REAL_TO_STR = std_format!("RealToStr"));
std_symbol!(STD_FORMAT_BOOL_TO_STR = std_format!("BoolToStr"));
std_symbol!(STD_FORMAT_INT_TO_HEX = std_format!("IntToHex"));

pub(in crate::std_units) const STD_FORMAT_SYMBOLS: &[&str] = &[
    STD_FORMAT_INT_TO_STR,
    STD_FORMAT_REAL_TO_STR,
    STD_FORMAT_BOOL_TO_STR,
    STD_FORMAT_INT_TO_HEX,
];
