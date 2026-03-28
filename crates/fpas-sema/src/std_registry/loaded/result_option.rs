//! Registration of `Std.Result` and `Std.Option` units.
//!
//! **Documentation:** `docs/pascal/std/result.md` and `docs/pascal/std/option.md` (from the repository root).

use super::super::define_builtin_std;
use crate::check::Checker;
use crate::types::{FunctionTy, Ty};
use fpas_std::std_symbols as s;

pub(super) fn register_std_result(checker: &mut Checker) {
    let placeholder = Ty::Function(FunctionTy {
        params: vec![],
        return_type: Box::new(Ty::Error),
    });
    for name in [
        s::STD_RESULT_UNWRAP,
        s::STD_RESULT_UNWRAP_OR,
        s::STD_RESULT_IS_OK,
        s::STD_RESULT_IS_ERR,
    ] {
        define_builtin_std(checker, name, placeholder.clone());
    }
}

pub(super) fn register_std_option(checker: &mut Checker) {
    let placeholder = Ty::Function(FunctionTy {
        params: vec![],
        return_type: Box::new(Ty::Error),
    });
    for name in [
        s::STD_OPTION_UNWRAP,
        s::STD_OPTION_UNWRAP_OR,
        s::STD_OPTION_IS_SOME,
        s::STD_OPTION_IS_NONE,
    ] {
        define_builtin_std(checker, name, placeholder.clone());
    }
}
