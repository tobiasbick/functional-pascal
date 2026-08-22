//! `Std.Parse` symbol names and registry group.

std_symbol!(STD_PARSE_TRY_INT = std_parse!("TryInt"));
std_symbol!(STD_PARSE_TRY_REAL = std_parse!("TryReal"));
std_symbol!(STD_PARSE_TRY_BOOL = std_parse!("TryBool"));

pub(in crate::std_units) const STD_PARSE_SYMBOLS: &[&str] =
    &[STD_PARSE_TRY_INT, STD_PARSE_TRY_REAL, STD_PARSE_TRY_BOOL];
