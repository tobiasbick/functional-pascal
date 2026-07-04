//! `Std.Array` symbol names and registry group.

pub const STD_ARRAY_LENGTH: &str = std_array!("Length");
pub const STD_ARRAY_SORT: &str = std_array!("Sort");
pub const STD_ARRAY_REVERSE: &str = std_array!("Reverse");
pub const STD_ARRAY_CONTAINS: &str = std_array!("Contains");
pub const STD_ARRAY_INDEX_OF: &str = std_array!("IndexOf");
pub const STD_ARRAY_SLICE: &str = std_array!("Slice");
pub const STD_ARRAY_PUSH: &str = std_array!("Push");
pub const STD_ARRAY_POP: &str = std_array!("Pop");
pub const STD_ARRAY_MAP: &str = std_array!("Map");
pub const STD_ARRAY_FILTER: &str = std_array!("Filter");
pub const STD_ARRAY_REDUCE: &str = std_array!("Reduce");
pub const STD_ARRAY_CONCAT: &str = std_array!("Concat");
pub const STD_ARRAY_FILL: &str = std_array!("Fill");
pub const STD_ARRAY_FIND: &str = std_array!("Find");
pub const STD_ARRAY_FIND_INDEX: &str = std_array!("FindIndex");
pub const STD_ARRAY_ANY: &str = std_array!("Any");
pub const STD_ARRAY_ALL: &str = std_array!("All");
pub const STD_ARRAY_FLAT_MAP: &str = std_array!("FlatMap");
pub const STD_ARRAY_FOR_EACH: &str = std_array!("ForEach");

pub(in crate::std_units) const STD_ARRAY_SYMBOLS: &[&str] = &[
    STD_ARRAY_LENGTH,
    STD_ARRAY_SORT,
    STD_ARRAY_REVERSE,
    STD_ARRAY_CONTAINS,
    STD_ARRAY_INDEX_OF,
    STD_ARRAY_SLICE,
    STD_ARRAY_PUSH,
    STD_ARRAY_POP,
    STD_ARRAY_MAP,
    STD_ARRAY_FILTER,
    STD_ARRAY_REDUCE,
    STD_ARRAY_CONCAT,
    STD_ARRAY_FILL,
    STD_ARRAY_FIND,
    STD_ARRAY_FIND_INDEX,
    STD_ARRAY_ANY,
    STD_ARRAY_ALL,
    STD_ARRAY_FLAT_MAP,
    STD_ARRAY_FOR_EACH,
];
