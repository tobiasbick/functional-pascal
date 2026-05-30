use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the `Std.Env` functions made visible by `uses Std.Env`.
pub(super) fn register_std_env(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_ENV_GET,
        vec![p("Name", Ty::String, false)],
        Ty::Option(Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_ENV_EXISTS,
        vec![p("Name", Ty::String, false)],
        Ty::Boolean,
    );
}
