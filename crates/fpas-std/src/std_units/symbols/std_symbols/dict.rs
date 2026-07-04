//! `Std.Dict` symbol names and registry group.

pub const STD_DICT_LENGTH: &str = std_dict!("Length");
pub const STD_DICT_CONTAINS_KEY: &str = std_dict!("ContainsKey");
pub const STD_DICT_KEYS: &str = std_dict!("Keys");
pub const STD_DICT_VALUES: &str = std_dict!("Values");
pub const STD_DICT_REMOVE: &str = std_dict!("Remove");
pub const STD_DICT_GET: &str = std_dict!("Get");
pub const STD_DICT_MERGE: &str = std_dict!("Merge");
pub const STD_DICT_MAP: &str = std_dict!("Map");
pub const STD_DICT_FILTER: &str = std_dict!("Filter");

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
