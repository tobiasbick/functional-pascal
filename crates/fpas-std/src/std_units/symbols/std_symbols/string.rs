//! `Std.Str` symbol names and registry group.

pub const STD_STR_LENGTH: &str = std_str!("Length");
pub const STD_STR_TO_UPPER: &str = std_str!("ToUpper");
pub const STD_STR_TO_LOWER: &str = std_str!("ToLower");
pub const STD_STR_TRIM: &str = std_str!("Trim");
pub const STD_STR_CONTAINS: &str = std_str!("Contains");
pub const STD_STR_STARTS_WITH: &str = std_str!("StartsWith");
pub const STD_STR_ENDS_WITH: &str = std_str!("EndsWith");
pub const STD_STR_SUBSTRING: &str = std_str!("Substring");
pub const STD_STR_INDEX_OF: &str = std_str!("IndexOf");
pub const STD_STR_REPLACE: &str = std_str!("Replace");
pub const STD_STR_SPLIT: &str = std_str!("Split");
pub const STD_STR_JOIN: &str = std_str!("Join");
pub const STD_STR_IS_NUMERIC: &str = std_str!("IsNumeric");
pub const STD_STR_REPEAT: &str = std_str!("RepeatStr");
pub const STD_STR_PAD_LEFT: &str = std_str!("PadLeft");
pub const STD_STR_PAD_RIGHT: &str = std_str!("PadRight");
pub const STD_STR_PAD_CENTER: &str = std_str!("PadCenter");
pub const STD_STR_FROM_CHAR: &str = std_str!("FromChar");
pub const STD_STR_CHAR_AT: &str = std_str!("CharAt");
pub const STD_STR_SET_CHAR_AT: &str = std_str!("SetCharAt");
pub const STD_STR_ORD: &str = std_str!("Ord");
pub const STD_STR_CHR: &str = std_str!("Chr");
pub const STD_STR_INSERT: &str = std_str!("Insert");
pub const STD_STR_DELETE: &str = std_str!("Delete");
pub const STD_STR_REVERSE: &str = std_str!("Reverse");
pub const STD_STR_TRIM_LEFT: &str = std_str!("TrimLeft");
pub const STD_STR_TRIM_RIGHT: &str = std_str!("TrimRight");
pub const STD_STR_LAST_INDEX_OF: &str = std_str!("LastIndexOf");
pub const STD_STR_FORMAT: &str = std_str!("Format");

pub(in crate::std_units) const STD_STR_SYMBOLS: &[&str] = &[
    STD_STR_LENGTH,
    STD_STR_TO_UPPER,
    STD_STR_TO_LOWER,
    STD_STR_TRIM,
    STD_STR_CONTAINS,
    STD_STR_STARTS_WITH,
    STD_STR_ENDS_WITH,
    STD_STR_SUBSTRING,
    STD_STR_INDEX_OF,
    STD_STR_REPLACE,
    STD_STR_SPLIT,
    STD_STR_JOIN,
    STD_STR_IS_NUMERIC,
    STD_STR_REPEAT,
    STD_STR_PAD_LEFT,
    STD_STR_PAD_RIGHT,
    STD_STR_PAD_CENTER,
    STD_STR_FROM_CHAR,
    STD_STR_CHAR_AT,
    STD_STR_SET_CHAR_AT,
    STD_STR_ORD,
    STD_STR_CHR,
    STD_STR_INSERT,
    STD_STR_DELETE,
    STD_STR_REVERSE,
    STD_STR_TRIM_LEFT,
    STD_STR_TRIM_RIGHT,
    STD_STR_LAST_INDEX_OF,
    STD_STR_FORMAT,
];
