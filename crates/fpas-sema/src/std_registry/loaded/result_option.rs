//! Registration of `Std.Result` and `Std.Option` units.
//!
//! **Documentation:** `docs/pascal/std/result.md` and `docs/pascal/std/option.md` (from the repository root).

use super::super::define_builtin_std;
use crate::check::Checker;
use crate::types::{FunctionTy, Ty};
use fpas_std::std_symbols as s;

pub(super) fn register_std_result(checker: &mut Checker) {
    let placeholder = Ty::Function(FunctionTy {
        type_params: Vec::new(),
        params: vec![],
        return_type: Box::new(Ty::Error),
    });
    for name in [
        s::STD_RESULT_UNWRAP,
        s::STD_RESULT_UNWRAP_OR,
        s::STD_RESULT_IS_OK,
        s::STD_RESULT_IS_ERR,
        s::STD_RESULT_MAP,
        s::STD_RESULT_AND_THEN,
        s::STD_RESULT_OR_ELSE,
    ] {
        define_builtin_std(checker, name, placeholder.clone());
    }
}

pub(super) fn register_std_option(checker: &mut Checker) {
    let placeholder = Ty::Function(FunctionTy {
        type_params: Vec::new(),
        params: vec![],
        return_type: Box::new(Ty::Error),
    });
    for name in [
        s::STD_OPTION_UNWRAP,
        s::STD_OPTION_UNWRAP_OR,
        s::STD_OPTION_IS_SOME,
        s::STD_OPTION_IS_NONE,
        s::STD_OPTION_MAP,
        s::STD_OPTION_AND_THEN,
        s::STD_OPTION_OR_ELSE,
    ] {
        define_builtin_std(checker, name, placeholder.clone());
    }
}
