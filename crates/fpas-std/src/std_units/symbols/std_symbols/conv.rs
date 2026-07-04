//! `Std.Conv` symbol names and registry group.

pub const STD_CONV_INT_TO_STR: &str = std_conv!("IntToStr");
pub const STD_CONV_STR_TO_INT: &str = std_conv!("StrToInt");
pub const STD_CONV_REAL_TO_STR: &str = std_conv!("RealToStr");
pub const STD_CONV_STR_TO_REAL: &str = std_conv!("StrToReal");
pub const STD_CONV_INT_TO_REAL: &str = std_conv!("IntToReal");
pub const STD_CONV_BOOL_TO_STR: &str = std_conv!("BoolToStr");
pub const STD_CONV_STR_TO_BOOL: &str = std_conv!("StrToBool");
pub const STD_CONV_INT_TO_HEX: &str = std_conv!("IntToHex");
pub const STD_CONV_HEX_TO_INT: &str = std_conv!("HexToInt");

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
