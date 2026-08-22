//! `Std.Str` symbol names and registry group.

std_symbol!(STD_STR_LENGTH = std_str!("Length"));
std_symbol!(STD_STR_TO_UPPER = std_str!("ToUpper"));
std_symbol!(STD_STR_TO_LOWER = std_str!("ToLower"));
std_symbol!(STD_STR_TRIM = std_str!("Trim"));
std_symbol!(STD_STR_CONTAINS = std_str!("Contains"));
std_symbol!(STD_STR_STARTS_WITH = std_str!("StartsWith"));
std_symbol!(STD_STR_ENDS_WITH = std_str!("EndsWith"));
std_symbol!(STD_STR_SUBSTRING = std_str!("Substring"));
std_symbol!(STD_STR_INDEX_OF = std_str!("IndexOf"));
std_symbol!(STD_STR_REPLACE = std_str!("Replace"));
std_symbol!(STD_STR_SPLIT = std_str!("Split"));
std_symbol!(STD_STR_JOIN = std_str!("Join"));
std_symbol!(STD_STR_IS_NUMERIC = std_str!("IsNumeric"));
std_symbol!(STD_STR_REPEAT = std_str!("RepeatStr"));
std_symbol!(STD_STR_PAD_LEFT = std_str!("PadLeft"));
std_symbol!(STD_STR_PAD_RIGHT = std_str!("PadRight"));
std_symbol!(STD_STR_PAD_CENTER = std_str!("PadCenter"));
std_symbol!(STD_STR_FROM_CHAR = std_str!("FromChar"));
std_symbol!(STD_STR_CHAR_AT = std_str!("CharAt"));
std_symbol!(STD_STR_SET_CHAR_AT = std_str!("SetCharAt"));
std_symbol!(STD_STR_ORD = std_str!("Ord"));
std_symbol!(STD_STR_CHR = std_str!("Chr"));
std_symbol!(STD_STR_INSERT = std_str!("Insert"));
std_symbol!(STD_STR_DELETE = std_str!("Delete"));
std_symbol!(STD_STR_REVERSE = std_str!("Reverse"));
std_symbol!(STD_STR_TRIM_LEFT = std_str!("TrimLeft"));
std_symbol!(STD_STR_TRIM_RIGHT = std_str!("TrimRight"));
std_symbol!(STD_STR_LAST_INDEX_OF = std_str!("LastIndexOf"));
std_symbol!(STD_STR_FORMAT = std_str!("Format"));

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
