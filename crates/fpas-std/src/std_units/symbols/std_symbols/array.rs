//! `Std.Array` symbol names and registry group.

std_symbol!(STD_ARRAY_LENGTH = std_array!("Length"));
std_symbol!(STD_ARRAY_SORT = std_array!("Sort"));
std_symbol!(STD_ARRAY_REVERSE = std_array!("Reverse"));
std_symbol!(STD_ARRAY_CONTAINS = std_array!("Contains"));
std_symbol!(STD_ARRAY_INDEX_OF = std_array!("IndexOf"));
std_symbol!(STD_ARRAY_SLICE = std_array!("Slice"));
std_symbol!(STD_ARRAY_PUSH = std_array!("Push"));
std_symbol!(STD_ARRAY_POP = std_array!("Pop"));
std_symbol!(STD_ARRAY_MAP = std_array!("Map"));
std_symbol!(STD_ARRAY_FILTER = std_array!("Filter"));
std_symbol!(STD_ARRAY_REDUCE = std_array!("Reduce"));
std_symbol!(STD_ARRAY_CONCAT = std_array!("Concat"));
std_symbol!(STD_ARRAY_FILL = std_array!("Fill"));
std_symbol!(STD_ARRAY_FIND = std_array!("Find"));
std_symbol!(STD_ARRAY_FIND_INDEX = std_array!("FindIndex"));
std_symbol!(STD_ARRAY_ANY = std_array!("Any"));
std_symbol!(STD_ARRAY_ALL = std_array!("All"));
std_symbol!(STD_ARRAY_FLAT_MAP = std_array!("FlatMap"));
std_symbol!(STD_ARRAY_FOR_EACH = std_array!("ForEach"));

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
