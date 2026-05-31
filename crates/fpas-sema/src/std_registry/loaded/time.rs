use super::super::{define_func, define_proc, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the `Std.Time` functions made visible by `uses Std.Time`.
pub(super) fn register_std_time(checker: &mut Checker) {
    define_func(checker, s::STD_TIME_TIMESTAMP_MILLIS, vec![], Ty::Integer);
    define_func(checker, s::STD_TIME_MONOTONIC_MILLIS, vec![], Ty::Integer);
    define_func(
        checker,
        s::STD_TIME_ELAPSED_MILLIS,
        vec![p("Start", Ty::Integer, false)],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_TIME_SLEEP,
        vec![p("Milliseconds", Ty::Integer, false)],
    );
}
