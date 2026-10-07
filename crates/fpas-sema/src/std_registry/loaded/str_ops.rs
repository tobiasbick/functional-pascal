use super::super::{define_builtin_std, define_func, define_func_variadic, p};
use crate::check::Checker;
use crate::types::{FunctionTy, Ty};
use fpas_std::std_symbols as s;

pub(super) fn register_std_str(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_STR_LENGTH,
        vec![p("S", Ty::String)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_STR_TO_UPPER,
        vec![p("S", Ty::String)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_TO_LOWER,
        vec![p("S", Ty::String)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_TRIM,
        vec![p("S", Ty::String)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_CONTAINS,
        vec![p("S", Ty::String), p("Sub", Ty::String)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_STR_STARTS_WITH,
        vec![p("S", Ty::String), p("Pre", Ty::String)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_STR_ENDS_WITH,
        vec![p("S", Ty::String), p("Suf", Ty::String)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_STR_SUBSTRING,
        vec![
            p("S", Ty::String),
            p("Start", Ty::Integer),
            p("Len", Ty::Integer),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_INDEX_OF,
        vec![p("S", Ty::String), p("Sub", Ty::String)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_STR_REPLACE,
        vec![
            p("S", Ty::String),
            p("Old", Ty::String),
            p("New", Ty::String),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_SPLIT,
        vec![p("S", Ty::String), p("Delim", Ty::String)],
        Ty::Array(Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_STR_JOIN,
        vec![
            p("Parts", Ty::Array(Box::new(Ty::String))),
            p("Delim", Ty::String),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_IS_NUMERIC,
        vec![p("S", Ty::String)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_STR_REPEAT,
        vec![p("S", Ty::String), p("N", Ty::Integer)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_PAD_LEFT,
        vec![
            p("S", Ty::String),
            p("Width", Ty::Integer),
            p("PadChar", Ty::String),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_PAD_RIGHT,
        vec![
            p("S", Ty::String),
            p("Width", Ty::Integer),
            p("PadChar", Ty::String),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_PAD_CENTER,
        vec![
            p("S", Ty::String),
            p("Width", Ty::Integer),
            p("PadChar", Ty::String),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_FROM_CHAR,
        vec![p("C", Ty::String), p("N", Ty::Integer)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_CHAR_AT,
        vec![p("S", Ty::String), p("Index", Ty::Integer)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_SET_CHAR_AT,
        vec![
            p("S", Ty::String),
            p("Index", Ty::Integer),
            p("C", Ty::String),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_ORD,
        vec![p("C", Ty::String)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_STR_CHR,
        vec![p("N", Ty::Integer)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_INSERT,
        vec![
            p("S", Ty::String),
            p("Index", Ty::Integer),
            p("Sub", Ty::String),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_DELETE,
        vec![
            p("S", Ty::String),
            p("Index", Ty::Integer),
            p("Len", Ty::Integer),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_REVERSE,
        vec![p("S", Ty::String)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_TRIM_LEFT,
        vec![p("S", Ty::String)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_TRIM_RIGHT,
        vec![p("S", Ty::String)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_STR_LAST_INDEX_OF,
        vec![p("S", Ty::String), p("Sub", Ty::String)],
        Ty::Integer,
    );
    define_func_variadic(
        checker,
        s::STD_STR_FORMAT,
        vec![p("Template", Ty::String)],
        Ty::String,
    );
    let callback_placeholder = Ty::Function(FunctionTy {
        type_params: Vec::new(),
        params: Vec::new(),
        return_type: Box::new(Ty::Error),
        variadic: false,
    });
    for name in [s::STD_STR_MAP, s::STD_STR_FILTER, s::STD_STR_REDUCE] {
        define_builtin_std(checker, name, callback_placeholder.clone());
    }
}
