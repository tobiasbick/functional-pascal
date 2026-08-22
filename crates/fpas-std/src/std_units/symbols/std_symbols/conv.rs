//! `Std.Conv` symbol names and registry group.

std_symbol!(STD_CONV_INT_TO_STR = std_conv!("IntToStr"));
std_symbol!(STD_CONV_STR_TO_INT = std_conv!("StrToInt"));
std_symbol!(STD_CONV_REAL_TO_STR = std_conv!("RealToStr"));
std_symbol!(STD_CONV_STR_TO_REAL = std_conv!("StrToReal"));
std_symbol!(STD_CONV_INT_TO_REAL = std_conv!("IntToReal"));
std_symbol!(STD_CONV_BOOL_TO_STR = std_conv!("BoolToStr"));
std_symbol!(STD_CONV_STR_TO_BOOL = std_conv!("StrToBool"));
std_symbol!(STD_CONV_INT_TO_HEX = std_conv!("IntToHex"));
std_symbol!(STD_CONV_HEX_TO_INT = std_conv!("HexToInt"));

pub(in crate::std_units) const STD_CONV_SYMBOLS: &[&str] = &[
    STD_CONV_INT_TO_STR,
    STD_CONV_STR_TO_INT,
    STD_CONV_REAL_TO_STR,
    STD_CONV_STR_TO_REAL,
    STD_CONV_INT_TO_REAL,
    STD_CONV_BOOL_TO_STR,
    STD_CONV_STR_TO_BOOL,
    STD_CONV_INT_TO_HEX,
    STD_CONV_HEX_TO_INT,
];
