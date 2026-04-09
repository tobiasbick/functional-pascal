//! Registration of `Std.Task`.
//!
//! **Documentation:** `docs/pascal/std/task.md` (from the repository root); language rules: `docs/pascal/08-concurrency.md`.

use super::super::define_builtin_std;
use crate::check::Checker;
use crate::types::{FunctionTy, Ty};
use fpas_std::std_symbols as s;

pub fn register_std_task(c: &mut Checker) {
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
