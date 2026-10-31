use super::super::{define_func, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_builtins::std_symbols as s;

/// Registers the four Result parsers documented in `docs/pascal/std/text/parse.md`.
pub(super) fn register_std_parse(checker: &mut Checker) {
    let error_ty = super::errors::register(
        checker,
        fpas_builtins::native_errors::NativeErrorFamily::Parse,
    );
    define_func(
        checker,
        s::STD_PARSE_INT,
        vec![p("Text", Ty::String)],
        Ty::Result(Box::new(Ty::Integer), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_PARSE_REAL,
        vec![p("Text", Ty::String)],
        Ty::Result(Box::new(Ty::Real), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_PARSE_BOOL,
        vec![p("Text", Ty::String)],
        Ty::Result(Box::new(Ty::Boolean), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_PARSE_HEX_INT,
        vec![p("Text", Ty::String)],
        Ty::Result(Box::new(Ty::Integer), Box::new(error_ty)),
    );
}
