//! Registration of `Std.Test` procedures.
//!
//! **Documentation:** `docs/pascal/std/testing/test.md` (from the repository root).

use super::super::builtins::register_assert_equals_builtin;
use super::super::{define_proc, p};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the `Std.Test` procedures made visible by `uses Std.Test`.
pub(super) fn register_std_test(checker: &mut Checker) {
    define_proc(
        checker,
        s::STD_TEST_ASSERT_TRUE,
        vec![p("Cond", Ty::Boolean, false)],
    );
    define_proc(
        checker,
        s::STD_TEST_ASSERT_FALSE,
        vec![p("Cond", Ty::Boolean, false)],
    );
    register_assert_equals_builtin(checker);
    define_proc(checker, s::STD_TEST_FAIL, vec![p("Msg", Ty::String, false)]);
    define_proc(checker, s::STD_TEST_SKIP, vec![p("Msg", Ty::String, false)]);
    define_proc(
        checker,
        s::STD_TEST_ASSERT_SCREEN_LINE,
        vec![p("Expected", Ty::String, false), p("Y", Ty::Integer, false)],
    );
    define_proc(
        checker,
        s::STD_TEST_ASSERT_SCREEN_CELL,
        vec![
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Ch", Ty::String, false),
            p("Fg", Ty::Integer, false),
            p("Bg", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TEST_PUSH_READLN,
        vec![p("Line", Ty::String, false)],
    );
}
