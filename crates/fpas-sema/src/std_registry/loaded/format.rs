use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_builtins::std_symbols as s;

/// Registers typed formatting from `docs/pascal/std/text/format.md`.
pub(super) fn register_std_format(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_FORMAT_INT_TO_STR,
        vec![p("N", Ty::Integer)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_FORMAT_REAL_TO_STR,
        vec![p("R", Ty::Real)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_FORMAT_BOOL_TO_STR,
        vec![p("B", Ty::Boolean)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_FORMAT_INT_TO_HEX,
        vec![p("N", Ty::Integer), p("Digits", Ty::Integer)],
        Ty::String,
    );
}
