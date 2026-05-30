use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

pub(super) fn register_std_args(checker: &mut Checker) {
    define_func(checker, s::STD_ARGS_PARAM_COUNT, vec![], Ty::Integer);
    define_func(
        checker,
        s::STD_ARGS_PARAM_STR,
        vec![p("Index", Ty::Integer, false)],
        Ty::String,
    );
}
