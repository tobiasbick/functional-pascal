//! `Std.Args` symbol names and registry group.

std_symbol!(STD_ARGS_PARAM_COUNT = std_args!("ParamCount"));
std_symbol!(STD_ARGS_PARAM_STR = std_args!("ParamStr"));

pub(in crate::std_units) const STD_ARGS_SYMBOLS: &[&str] =
    &[STD_ARGS_PARAM_COUNT, STD_ARGS_PARAM_STR];
