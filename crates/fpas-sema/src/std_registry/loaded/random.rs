use super::super::{define_func, define_proc, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

pub(super) fn register_std_random(checker: &mut Checker) {
    define_func(checker, s::STD_RANDOM_RANDOM, vec![], Ty::Real);
    define_func(
        checker,
        s::STD_RANDOM_RANDOM_INT,
        vec![p("Lo", Ty::Integer, false), p("Hi", Ty::Integer, false)],
        Ty::Integer,
    );
    define_proc(checker, s::STD_RANDOM_RANDOMIZE, vec![]);
}
