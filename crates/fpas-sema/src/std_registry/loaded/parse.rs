use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Registers the four Result parsers documented in `docs/pascal/std/text/parse.md`.
pub(super) fn register_std_parse(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_PARSE_INT,
        vec![p("Text", Ty::String)],
        Ty::Result(Box::new(Ty::Integer), Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_PARSE_REAL,
        vec![p("Text", Ty::String)],
        Ty::Result(Box::new(Ty::Real), Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_PARSE_BOOL,
        vec![p("Text", Ty::String)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_PARSE_HEX_INT,
        vec![p("Text", Ty::String)],
        Ty::Result(Box::new(Ty::Integer), Box::new(Ty::String)),
    );
}
