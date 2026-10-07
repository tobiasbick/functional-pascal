use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

pub(super) fn register_std_conv(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_CONV_INT_TO_STR,
        vec![p("N", Ty::Integer)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_CONV_STR_TO_INT,
        vec![p("S", Ty::String)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_CONV_REAL_TO_STR,
        vec![p("R", Ty::Real)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_CONV_STR_TO_REAL,
        vec![p("S", Ty::String)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_CONV_INT_TO_REAL,
        vec![p("N", Ty::Integer)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_CONV_BOOL_TO_STR,
        vec![p("B", Ty::Boolean)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_CONV_STR_TO_BOOL,
        vec![p("S", Ty::String)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_CONV_INT_TO_HEX,
        vec![p("N", Ty::Integer), p("Digits", Ty::Integer)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_CONV_HEX_TO_INT,
        vec![p("S", Ty::String)],
        Ty::Integer,
    );
}
