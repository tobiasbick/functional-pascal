//! `Std.Dict` symbol names and registry group.

std_symbol!(STD_DICT_LENGTH = std_dict!("Length"));
std_symbol!(STD_DICT_CONTAINS_KEY = std_dict!("ContainsKey"));
std_symbol!(STD_DICT_KEYS = std_dict!("Keys"));
std_symbol!(STD_DICT_VALUES = std_dict!("Values"));
std_symbol!(STD_DICT_REMOVE = std_dict!("Remove"));
std_symbol!(STD_DICT_GET = std_dict!("Get"));
std_symbol!(STD_DICT_MERGE = std_dict!("Merge"));
std_symbol!(STD_DICT_MAP = std_dict!("Map"));
std_symbol!(STD_DICT_FILTER = std_dict!("Filter"));

pub(in crate::std_units) const STD_DICT_SYMBOLS: &[&str] = &[
    STD_DICT_LENGTH,
    STD_DICT_CONTAINS_KEY,
    STD_DICT_KEYS,
    STD_DICT_VALUES,
    STD_DICT_REMOVE,
    STD_DICT_GET,
    STD_DICT_MERGE,
    STD_DICT_MAP,
    STD_DICT_FILTER,
];
