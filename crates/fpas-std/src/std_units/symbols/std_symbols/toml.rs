//! `Std.Toml` symbol names and registry group.

std_symbol!(STD_TOML_VALUE = std_toml!("TomlValue"));
std_symbol!(STD_TOML_VALUE_STRING = std_toml!("TomlValue.String"));
std_symbol!(STD_TOML_VALUE_INTEGER = std_toml!("TomlValue.Integer"));
std_symbol!(STD_TOML_VALUE_FLOAT = std_toml!("TomlValue.Float"));
std_symbol!(STD_TOML_VALUE_BOOLEAN = std_toml!("TomlValue.Boolean"));
std_symbol!(STD_TOML_VALUE_DATETIME = std_toml!("TomlValue.Datetime"));
std_symbol!(STD_TOML_VALUE_ARRAY = std_toml!("TomlValue.Array"));
std_symbol!(STD_TOML_VALUE_TABLE = std_toml!("TomlValue.Table"));
std_symbol!(STD_TOML_PARSE = std_toml!("Parse"));
std_symbol!(STD_TOML_STRINGIFY = std_toml!("Stringify"));

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
