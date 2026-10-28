//! `Std.Text.Parse` symbol names and registry group.

std_symbol!(STD_PARSE_INT = std_parse!("Int"));
std_symbol!(STD_PARSE_REAL = std_parse!("Real"));
std_symbol!(STD_PARSE_BOOL = std_parse!("Bool"));
std_symbol!(STD_PARSE_HEX_INT = std_parse!("HexInt"));

pub(in crate::std_units) const STD_PARSE_SYMBOLS: &[&str] = &[
    STD_PARSE_INT,
    STD_PARSE_REAL,
    STD_PARSE_BOOL,
    STD_PARSE_HEX_INT,
];
