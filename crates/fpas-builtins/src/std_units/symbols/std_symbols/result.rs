//! `Std.Results` symbol names and registry group.

std_symbol!(STD_RESULT_UNWRAP = std_result!("Unwrap"));
std_symbol!(STD_RESULT_MAP = std_result!("Map"));
std_symbol!(STD_RESULT_MAP_ERROR = std_result!("MapError"));
std_symbol!(STD_RESULT_AND_THEN = std_result!("AndThen"));
std_symbol!(STD_RESULT_OR_ELSE = std_result!("OrElse"));

pub(in crate::std_units) const STD_RESULT_SYMBOLS: &[&str] = &[
    STD_RESULT_UNWRAP,
    STD_RESULT_MAP,
    STD_RESULT_MAP_ERROR,
    STD_RESULT_AND_THEN,
    STD_RESULT_OR_ELSE,
];
