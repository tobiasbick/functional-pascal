use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the `Std.Fs` functions made visible by `uses Std.Fs`.
pub(super) fn register_std_fs(checker: &mut Checker) {
    let error_ty = super::errors::register(checker, fpas_std::native_errors::NativeErrorFamily::Fs);
    define_func(
        checker,
        s::STD_FS_READ_TEXT,
        vec![p("Path", Ty::String)],
        Ty::Result(Box::new(Ty::String), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_FS_WRITE_TEXT,
        vec![p("Path", Ty::String), p("Text", Ty::String)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_FS_WRITE_TEXT_ATOMIC,
        vec![p("Path", Ty::String), p("Text", Ty::String)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_FS_DELETE_FILE,
        vec![p("Path", Ty::String)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(error_ty.clone())),
    );
    for symbol in [s::STD_FS_DELETE_DIR, s::STD_FS_DELETE_DIR_ALL] {
        define_func(
            checker,
            symbol,
            vec![p("Path", Ty::String)],
            Ty::Result(Box::new(Ty::Boolean), Box::new(error_ty.clone())),
        );
    }
    for symbol in [s::STD_FS_FILE_SIZE, s::STD_FS_MODIFIED_TIME] {
        define_func(
            checker,
            symbol,
            vec![p("Path", Ty::String)],
            Ty::Result(Box::new(Ty::Integer), Box::new(error_ty.clone())),
        );
    }
    define_func(
        checker,
        s::STD_FS_IS_SYM_LINK,
        vec![p("Path", Ty::String)],
        Ty::Boolean,
    );
    for symbol in [
        s::STD_FS_CURRENT_DIR,
        s::STD_FS_CREATE_TEMP_DIR,
        s::STD_FS_CREATE_TEMP_FILE,
    ] {
        define_func(
            checker,
            symbol,
            vec![],
            Ty::Result(Box::new(Ty::String), Box::new(error_ty.clone())),
        );
    }
    define_func(checker, s::STD_FS_TEMP_DIR, vec![], Ty::String);
    define_func(
        checker,
        s::STD_FS_SET_CURRENT_DIR,
        vec![p("Path", Ty::String)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_FS_READ_DIR_ALL,
        vec![p("Path", Ty::String)],
        Ty::Result(
            Box::new(Ty::Array(Box::new(Ty::String))),
            Box::new(error_ty.clone()),
        ),
    );
    for symbol in [
        s::STD_FS_RENAME,
        s::STD_FS_COPY_FILE,
        s::STD_FS_COPY_DIR_ALL,
    ] {
        define_func(
            checker,
            symbol,
            vec![p("Source", Ty::String), p("Target", Ty::String)],
            Ty::Result(Box::new(Ty::Boolean), Box::new(error_ty.clone())),
        );
    }
    define_func(
        checker,
        s::STD_FS_APPEND_TEXT,
        vec![p("Path", Ty::String), p("Text", Ty::String)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_FS_EXISTS,
        vec![p("Path", Ty::String)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_FS_IS_FILE,
        vec![p("Path", Ty::String)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_FS_IS_DIR,
        vec![p("Path", Ty::String)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_FS_CREATE_DIR,
        vec![p("Path", Ty::String)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_FS_CREATE_DIR_ALL,
        vec![p("Path", Ty::String)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_FS_READ_DIR,
        vec![p("Path", Ty::String)],
        Ty::Result(
            Box::new(Ty::Array(Box::new(Ty::String))),
            Box::new(error_ty.clone()),
        ),
    );
    define_func(
        checker,
        s::STD_FS_GLOB,
        vec![p("Pattern", Ty::String)],
        Ty::Result(
            Box::new(Ty::Array(Box::new(Ty::String))),
            Box::new(error_ty.clone()),
        ),
    );
}
