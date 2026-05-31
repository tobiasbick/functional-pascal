use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the `Std.Path` functions made visible by `uses Std.Path`.
pub(super) fn register_std_path(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_PATH_JOIN,
        vec![p("Segments", Ty::Array(Box::new(Ty::String)), false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_PATH_BASE_NAME,
        vec![p("Path", Ty::String, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_PATH_DIR_NAME,
        vec![p("Path", Ty::String, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_PATH_EXTENSION,
        vec![p("Path", Ty::String, false)],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_PATH_NORMALIZE,
        vec![p("Path", Ty::String, false)],
        Ty::String,
    );
}
