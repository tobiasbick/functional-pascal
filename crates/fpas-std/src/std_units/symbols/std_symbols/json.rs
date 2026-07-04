//! `Std.Json` symbol names and registry group.

pub const STD_JSON_VALUE: &str = std_json!("JsonValue");
pub const STD_JSON_VALUE_NULL: &str = std_json!("JsonValue.Null");
pub const STD_JSON_VALUE_BOOL: &str = std_json!("JsonValue.Bool");
pub const STD_JSON_VALUE_NUMBER: &str = std_json!("JsonValue.Number");
pub const STD_JSON_VALUE_STRING: &str = std_json!("JsonValue.String");
pub const STD_JSON_VALUE_ARRAY: &str = std_json!("JsonValue.Array");
pub const STD_JSON_VALUE_OBJECT: &str = std_json!("JsonValue.Object");
pub const STD_JSON_PARSE: &str = std_json!("Parse");
pub const STD_JSON_STRINGIFY: &str = std_json!("Stringify");

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
