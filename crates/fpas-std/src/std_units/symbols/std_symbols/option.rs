//! `Std.Option` symbol names and registry group.

pub const STD_OPTION_UNWRAP: &str = std_option!("Unwrap");
pub const STD_OPTION_UNWRAP_OR: &str = std_option!("UnwrapOr");
pub const STD_OPTION_IS_SOME: &str = std_option!("IsSome");
pub const STD_OPTION_IS_NONE: &str = std_option!("IsNone");
pub const STD_OPTION_MAP: &str = std_option!("Map");
pub const STD_OPTION_AND_THEN: &str = std_option!("AndThen");
pub const STD_OPTION_OR_ELSE: &str = std_option!("OrElse");

pub(in crate::std_units) const STD_OPTION_SYMBOLS: &[&str] = &[
    STD_OPTION_UNWRAP,
    STD_OPTION_UNWRAP_OR,
    STD_OPTION_IS_SOME,
    STD_OPTION_IS_NONE,
    STD_OPTION_MAP,
    STD_OPTION_AND_THEN,
    STD_OPTION_OR_ELSE,
];
