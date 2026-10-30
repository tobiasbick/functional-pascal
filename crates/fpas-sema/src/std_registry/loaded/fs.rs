use super::super::{define_func, p};
use super::type_registration;
use crate::check::Checker;
use crate::types::Ty;
use fpas_builtins::std_symbols as s;

/// Register the `Std.Io.Fs` functions made visible by `uses Std.Io.Fs`.
pub(super) fn register_std_fs(checker: &mut Checker) {
    let error_ty =
        super::errors::register(checker, fpas_builtins::native_errors::NativeErrorFamily::Fs);
    let file = type_registration::register_opaque_type(checker, s::STD_FS_FILE);
    let mode = type_registration::register_enum_type(
        checker,
        s::STD_FS_FILE_MODE,
        &["Read", "Create", "Append", "ReadWrite"],
    );
    let origin = type_registration::register_enum_type(
        checker,
        s::STD_FS_SEEK_ORIGIN,
        &["Start", "Current", "FileEnd"],
    );
    let token = type_registration::register_record_type(
        checker,
        s::STD_TASK_CANCELLATION_TOKEN,
        Vec::new(),
    );
    let bytes = Ty::ByteBuffer;
    let result = |ty| Ty::Result(Box::new(ty), Box::new(error_ty.clone()));
    define_func(
        checker,
        s::STD_FS_SIZE,
        vec![p("File", file.clone())],
        result(Ty::Integer),
    );
    define_func(
        checker,
        s::STD_FS_SET_LENGTH,
        vec![p("File", file.clone()), p("Length", Ty::Integer)],
        result(Ty::Boolean),
    );
    for (name, cancellable) in [
        (s::STD_FS_WRITE_STRING, false),
        (s::STD_FS_WRITE_LINE, false),
        (s::STD_FS_WRITE_STRING_WITH_CANCELLATION, true),
        (s::STD_FS_WRITE_LINE_WITH_CANCELLATION, true),
    ] {
        let mut parameters = vec![p("File", file.clone()), p("Text", Ty::String)];
        if cancellable {
            parameters.push(p("Token", token.clone()));
        }
        define_func(checker, name, parameters, result(Ty::Boolean));
    }
    define_func(
        checker,
        s::STD_FS_OPEN,
        vec![p("Path", Ty::String), p("Mode", mode)],
        result(file.clone()),
    );
    for name in [s::STD_FS_CLOSE, s::STD_FS_FLUSH] {
        define_func(
            checker,
            name,
            vec![p("File", file.clone())],
            result(Ty::Boolean),
        );
    }
    define_func(
        checker,
        s::STD_FS_READ,
        vec![p("File", file.clone()), p("MaxBytes", Ty::Integer)],
        result(bytes.clone()),
    );
    define_func(
        checker,
        s::STD_FS_WRITE,
        vec![p("File", file.clone()), p("Bytes", bytes.clone())],
        result(Ty::Integer),
    );
    define_func(
        checker,
        s::STD_FS_READ_LINE,
        vec![p("File", file.clone())],
        result(Ty::Option(Box::new(Ty::String))),
    );
    define_func(
        checker,
        s::STD_FS_SEEK,
        vec![
            p("File", file.clone()),
            p("Offset", Ty::Integer),
            p("Origin", origin),
        ],
        result(Ty::Integer),
    );
    define_func(
        checker,
        s::STD_FS_TELL,
        vec![p("File", file.clone())],
        result(Ty::Integer),
    );
    define_func(
        checker,
        s::STD_FS_READ_WITH_CANCELLATION,
        vec![
            p("File", file.clone()),
            p("MaxBytes", Ty::Integer),
            p("Token", token.clone()),
        ],
        result(bytes.clone()),
    );
    define_func(
        checker,
        s::STD_FS_WRITE_WITH_CANCELLATION,
        vec![
            p("File", file.clone()),
            p("Bytes", bytes),
            p("Token", token.clone()),
        ],
        result(Ty::Integer),
    );
    define_func(
        checker,
        s::STD_FS_READ_LINE_WITH_CANCELLATION,
        vec![p("File", file), p("Token", token)],
        result(Ty::Option(Box::new(Ty::String))),
    );

    define_func(
        checker,
        s::STD_FS_READ_BYTES,
        vec![p("Path", Ty::String)],
        Ty::Result(Box::new(Ty::ByteBuffer), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_FS_WRITE_BYTES,
        vec![p("Path", Ty::String), p("Bytes", Ty::ByteBuffer)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(error_ty.clone())),
    );
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
