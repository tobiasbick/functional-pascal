use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

pub(super) fn register_std_conv(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_CONV_INT_TO_STR,
        vec![p("N", Ty::Integer, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_CONV_STR_TO_INT,
        vec![p("S", Ty::String, false)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_CONV_REAL_TO_STR,
        vec![p("R", Ty::Real, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_CONV_STR_TO_REAL,
        vec![p("S", Ty::String, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_CONV_CHAR_TO_STR,
        vec![p("C", Ty::Char, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_CONV_INT_TO_REAL,
        vec![p("N", Ty::Integer, false)],
        Ty::Real,
    );
}
