//! `Std.Option` symbol names and registry group.

std_symbol!(STD_OPTION_UNWRAP = std_option!("Unwrap"));
std_symbol!(STD_OPTION_UNWRAP_OR = std_option!("UnwrapOr"));
std_symbol!(STD_OPTION_IS_SOME = std_option!("IsSome"));
std_symbol!(STD_OPTION_IS_NONE = std_option!("IsNone"));
std_symbol!(STD_OPTION_MAP = std_option!("Map"));
std_symbol!(STD_OPTION_AND_THEN = std_option!("AndThen"));
std_symbol!(STD_OPTION_OR_ELSE = std_option!("OrElse"));

pub(in crate::std_units) const STD_OPTION_SYMBOLS: &[&str] = &[
    STD_OPTION_UNWRAP,
    STD_OPTION_UNWRAP_OR,
    STD_OPTION_IS_SOME,
    STD_OPTION_IS_NONE,
    STD_OPTION_MAP,
    STD_OPTION_AND_THEN,
    STD_OPTION_OR_ELSE,
];
