//! Registration of `Std.Test` procedures.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

use super::super::builtins::register_assert_equals_builtin;
use super::super::{define_proc, p};
use super::type_registration;
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
            p("Ch", Ty::Char, false),
            p("Fg", Ty::Integer, false),
            p("Bg", Ty::Integer, false),
        ],
    );
    let application = if checker.scopes.lookup(s::STD_TUI_APPLICATION).is_some() {
        type_registration::lookup_required_type(checker, s::STD_TUI_APPLICATION, "Application")
    } else {
        type_registration::register_record_type(checker, s::STD_TUI_APPLICATION, Vec::new())
    };
    define_proc(
        checker,
        s::STD_TEST_ASSERT_VIEW_RECT,
        vec![
            p("App", application, false),
            p("V", Ty::Integer, false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("W", Ty::Integer, false),
            p("H", Ty::Integer, false),
        ],
    );
}
