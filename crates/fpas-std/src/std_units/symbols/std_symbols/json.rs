//! `Std.Json` symbol names and registry group.

std_symbol!(STD_JSON_VALUE = std_json!("JsonValue"));
std_symbol!(STD_JSON_VALUE_NULL = std_json!("JsonValue.Null"));
std_symbol!(STD_JSON_VALUE_BOOL = std_json!("JsonValue.Bool"));
std_symbol!(STD_JSON_VALUE_NUMBER = std_json!("JsonValue.Number"));
std_symbol!(STD_JSON_VALUE_STRING = std_json!("JsonValue.String"));
std_symbol!(STD_JSON_VALUE_ARRAY = std_json!("JsonValue.Array"));
std_symbol!(STD_JSON_VALUE_OBJECT = std_json!("JsonValue.Object"));
std_symbol!(STD_JSON_PARSE = std_json!("Parse"));
std_symbol!(STD_JSON_STRINGIFY = std_json!("Stringify"));

pub(in crate::std_units) const STD_JSON_SYMBOLS: &[&str] = &[
    STD_JSON_VALUE,
    STD_JSON_VALUE_NULL,
    STD_JSON_VALUE_BOOL,
    STD_JSON_VALUE_NUMBER,
    STD_JSON_VALUE_STRING,
    STD_JSON_VALUE_ARRAY,
    STD_JSON_VALUE_OBJECT,
    STD_JSON_PARSE,
    STD_JSON_STRINGIFY,
];
