//! `Std.Result` symbol names and registry group.

pub const STD_RESULT_UNWRAP: &str = std_result!("Unwrap");
pub const STD_RESULT_UNWRAP_OR: &str = std_result!("UnwrapOr");
pub const STD_RESULT_IS_OK: &str = std_result!("IsOk");
pub const STD_RESULT_IS_ERR: &str = std_result!("IsError");
pub const STD_RESULT_MAP: &str = std_result!("Map");
pub const STD_RESULT_AND_THEN: &str = std_result!("AndThen");
pub const STD_RESULT_OR_ELSE: &str = std_result!("OrElse");

pub(in crate::std_units) const STD_RESULT_SYMBOLS: &[&str] = &[
    STD_RESULT_UNWRAP,
    STD_RESULT_UNWRAP_OR,
    STD_RESULT_IS_OK,
    STD_RESULT_IS_ERR,
    STD_RESULT_MAP,
    STD_RESULT_AND_THEN,
    STD_RESULT_OR_ELSE,
];
