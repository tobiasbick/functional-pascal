//! Registration of `Std.Task`.
//!
//! **Documentation:** `docs/pascal/std/concurrency/task.md` (from the repository root); language rules: `docs/pascal/language/concurrency/README.md`.

use super::super::{define_builtin_std, define_func, p};
use super::type_registration;
use crate::check::Checker;
use crate::types::{FunctionTy, Ty};
use fpas_std::std_symbols as s;

pub fn register_std_task(c: &mut Checker) {
    let source =
        type_registration::register_record_type(c, s::STD_TASK_CANCELLATION_SOURCE, Vec::new());
    let token =
        type_registration::register_record_type(c, s::STD_TASK_CANCELLATION_TOKEN, Vec::new());

    define_func(
        c,
        s::STD_TASK_CREATE_CANCELLATION_SOURCE,
        vec![],
        source.clone(),
    );
    define_func(
        c,
        s::STD_TASK_GET_CANCELLATION_TOKEN,
        vec![p("Source", source.clone(), false)],
        token.clone(),
    );
    define_func(
        c,
        s::STD_TASK_CANCEL,
        vec![p("Source", source, false)],
        Ty::Boolean,
    );
    define_func(
        c,
        s::STD_TASK_IS_CANCELLATION_REQUESTED,
        vec![p("Token", token, false)],
        Ty::Boolean,
    );

    let placeholder = Ty::Function(FunctionTy {
        type_params: Vec::new(),
        params: vec![],
        return_type: Box::new(Ty::Error),
        variadic: false,
    });

    for name in [s::STD_TASK_WAIT, s::STD_TASK_WAIT_ALL] {
        define_builtin_std(c, name, placeholder.clone());
    }
}
