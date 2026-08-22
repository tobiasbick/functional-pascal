//! `Std.Env` symbol names and registry group.

std_symbol!(STD_ENV_GET = std_env!("Get"));
std_symbol!(STD_ENV_EXISTS = std_env!("Exists"));

pub(in crate::std_units) const STD_ENV_SYMBOLS: &[&str] = &[STD_ENV_GET, STD_ENV_EXISTS];
