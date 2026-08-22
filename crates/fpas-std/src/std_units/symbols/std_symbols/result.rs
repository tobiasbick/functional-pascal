//! `Std.Result` symbol names and registry group.

std_symbol!(STD_RESULT_UNWRAP = std_result!("Unwrap"));
std_symbol!(STD_RESULT_UNWRAP_OR = std_result!("UnwrapOr"));
std_symbol!(STD_RESULT_IS_OK = std_result!("IsOk"));
std_symbol!(STD_RESULT_IS_ERR = std_result!("IsError"));
std_symbol!(STD_RESULT_MAP = std_result!("Map"));
std_symbol!(STD_RESULT_AND_THEN = std_result!("AndThen"));
std_symbol!(STD_RESULT_OR_ELSE = std_result!("OrElse"));

pub(in crate::std_units) const STD_RESULT_SYMBOLS: &[&str] = &[
    STD_RESULT_UNWRAP,
    STD_RESULT_UNWRAP_OR,
    STD_RESULT_IS_OK,
    STD_RESULT_IS_ERR,
    STD_RESULT_MAP,
    STD_RESULT_AND_THEN,
    STD_RESULT_OR_ELSE,
];
