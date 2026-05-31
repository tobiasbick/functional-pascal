use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

pub(super) fn register_std_parse(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_PARSE_TRY_INT,
        vec![p("Text", Ty::String, false)],
        Ty::Result(Box::new(Ty::Integer), Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_PARSE_TRY_REAL,
        vec![p("Text", Ty::String, false)],
        Ty::Result(Box::new(Ty::Real), Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_PARSE_TRY_BOOL,
        vec![p("Text", Ty::String, false)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(Ty::String)),
    );
}
