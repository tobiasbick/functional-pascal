//! `Std.Env` symbol names and registry group.

pub const STD_ENV_GET: &str = std_env!("Get");
pub const STD_ENV_EXISTS: &str = std_env!("Exists");

pub(in crate::std_units) const STD_ENV_SYMBOLS: &[&str] = &[STD_ENV_GET, STD_ENV_EXISTS];
