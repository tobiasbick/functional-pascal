//! `Std.Toml` symbol names and registry group.

pub const STD_TOML_VALUE: &str = std_toml!("TomlValue");
pub const STD_TOML_VALUE_STRING: &str = std_toml!("TomlValue.String");
pub const STD_TOML_VALUE_INTEGER: &str = std_toml!("TomlValue.Integer");
pub const STD_TOML_VALUE_FLOAT: &str = std_toml!("TomlValue.Float");
pub const STD_TOML_VALUE_BOOLEAN: &str = std_toml!("TomlValue.Boolean");
pub const STD_TOML_VALUE_DATETIME: &str = std_toml!("TomlValue.Datetime");
pub const STD_TOML_VALUE_ARRAY: &str = std_toml!("TomlValue.Array");
pub const STD_TOML_VALUE_TABLE: &str = std_toml!("TomlValue.Table");
pub const STD_TOML_PARSE: &str = std_toml!("Parse");
pub const STD_TOML_STRINGIFY: &str = std_toml!("Stringify");

pub(in crate::std_units) const STD_TOML_SYMBOLS: &[&str] = &[
    STD_TOML_VALUE,
    STD_TOML_VALUE_STRING,
    STD_TOML_VALUE_INTEGER,
    STD_TOML_VALUE_FLOAT,
    STD_TOML_VALUE_BOOLEAN,
    STD_TOML_VALUE_DATETIME,
    STD_TOML_VALUE_ARRAY,
    STD_TOML_VALUE_TABLE,
    STD_TOML_PARSE,
    STD_TOML_STRINGIFY,
];
