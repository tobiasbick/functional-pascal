use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

pub(super) fn register_std_str(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_STR_LENGTH,
        vec![p("S", Ty::String, false)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_STR_TO_UPPER,
        vec![p("S", Ty::String, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_TO_LOWER,
        vec![p("S", Ty::String, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_TRIM,
        vec![p("S", Ty::String, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_CONTAINS,
        vec![p("S", Ty::String, false), p("Sub", Ty::String, false)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_STR_STARTS_WITH,
        vec![p("S", Ty::String, false), p("Pre", Ty::String, false)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_STR_ENDS_WITH,
        vec![p("S", Ty::String, false), p("Suf", Ty::String, false)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_STR_SUBSTRING,
        vec![
            p("S", Ty::String, false),
            p("Start", Ty::Integer, false),
            p("Len", Ty::Integer, false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_INDEX_OF,
        vec![p("S", Ty::String, false), p("Sub", Ty::String, false)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_STR_REPLACE,
        vec![
            p("S", Ty::String, false),
            p("Old", Ty::String, false),
            p("New", Ty::String, false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_SPLIT,
        vec![p("S", Ty::String, false), p("Delim", Ty::String, false)],
        Ty::Array(Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_STR_JOIN,
        vec![
            p("Parts", Ty::Array(Box::new(Ty::String)), false),
            p("Delim", Ty::String, false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_IS_NUMERIC,
        vec![p("S", Ty::String, false)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_STR_REPEAT,
        vec![p("S", Ty::String, false), p("N", Ty::Integer, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_PAD_LEFT,
        vec![
            p("S", Ty::String, false),
            p("Width", Ty::Integer, false),
            p("PadChar", Ty::Char, false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_PAD_RIGHT,
        vec![
            p("S", Ty::String, false),
            p("Width", Ty::Integer, false),
            p("PadChar", Ty::Char, false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_PAD_CENTER,
        vec![
            p("S", Ty::String, false),
            p("Width", Ty::Integer, false),
            p("PadChar", Ty::Char, false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_FROM_CHAR,
        vec![p("C", Ty::Char, false), p("N", Ty::Integer, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_CHAR_AT,
        vec![p("S", Ty::String, false), p("Index", Ty::Integer, false)],
        Ty::Char,
    );
    define_func(
        checker,
        s::STD_STR_SET_CHAR_AT,
        vec![
            p("S", Ty::String, false),
            p("Index", Ty::Integer, false),
            p("C", Ty::Char, false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_ORD,
        vec![p("C", Ty::Char, false)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_STR_CHR,
        vec![p("N", Ty::Integer, false)],
        Ty::Char,
    );
    define_func(
        checker,
        s::STD_STR_INSERT,
        vec![
            p("S", Ty::String, false),
            p("Index", Ty::Integer, false),
            p("Sub", Ty::String, false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_DELETE,
        vec![
            p("S", Ty::String, false),
            p("Index", Ty::Integer, false),
            p("Len", Ty::Integer, false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_REVERSE,
        vec![p("S", Ty::String, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_TRIM_LEFT,
        vec![p("S", Ty::String, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_TRIM_RIGHT,
        vec![p("S", Ty::String, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_LAST_INDEX_OF,
        vec![p("S", Ty::String, false), p("Sub", Ty::String, false)],
        Ty::Integer,
    );
}
