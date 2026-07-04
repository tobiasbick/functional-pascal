//! `Std.Args` symbol names and registry group.

pub const STD_ARGS_PARAM_COUNT: &str = std_args!("ParamCount");
pub const STD_ARGS_PARAM_STR: &str = std_args!("ParamStr");

pub(in crate::std_units) const STD_ARGS_SYMBOLS: &[&str] =
    &[STD_ARGS_PARAM_COUNT, STD_ARGS_PARAM_STR];
