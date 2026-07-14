use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the `Std.Fs` functions made visible by `uses Std.Fs`.
pub(super) fn register_std_fs(checker: &mut Checker) {
    define_func(
        checker,
        s::STD_FS_READ_TEXT,
        vec![p("Path", Ty::String, false)],
        Ty::Result(Box::new(Ty::String), Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_FS_WRITE_TEXT,
        vec![p("Path", Ty::String, false), p("Text", Ty::String, false)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_FS_EXISTS,
        vec![p("Path", Ty::String, false)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_FS_IS_FILE,
        vec![p("Path", Ty::String, false)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_FS_IS_DIR,
        vec![p("Path", Ty::String, false)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_FS_CREATE_DIR,
        vec![p("Path", Ty::String, false)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_FS_GLOB,
        vec![p("Pattern", Ty::String, false)],
        Ty::Result(
            Box::new(Ty::Array(Box::new(Ty::String))),
            Box::new(Ty::String),
        ),
    );
}
