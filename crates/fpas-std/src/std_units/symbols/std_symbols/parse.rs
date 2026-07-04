//! `Std.Parse` symbol names and registry group.

pub const STD_PARSE_TRY_INT: &str = std_parse!("TryInt");
pub const STD_PARSE_TRY_REAL: &str = std_parse!("TryReal");
pub const STD_PARSE_TRY_BOOL: &str = std_parse!("TryBool");

pub(in crate::std_units) const STD_PARSE_SYMBOLS: &[&str] =
    &[STD_PARSE_TRY_INT, STD_PARSE_TRY_REAL, STD_PARSE_TRY_BOOL];
