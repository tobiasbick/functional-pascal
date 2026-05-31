use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the `Std.Proc` functions made visible by `uses Std.Proc`.
pub(super) fn register_std_proc(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_PROC_RUN,
        vec![
            p("Command", Ty::String, false),
            p("Args", Ty::Array(Box::new(Ty::String)), false),
        ],
        Ty::Result(Box::new(Ty::Integer), Box::new(Ty::String)),
    );
}
