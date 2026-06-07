//! Registration of `Std.Test` procedures.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

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
    define_proc(
        checker,
        s::STD_TEST_ASSERT_EQUALS,
        vec![
            p("Expected", Ty::Integer, false),
            p("Actual", Ty::Integer, false),
        ],
    );
    define_proc(checker, s::STD_TEST_FAIL, vec![p("Msg", Ty::String, false)]);
    define_proc(checker, s::STD_TEST_SKIP, vec![p("Msg", Ty::String, false)]);
}
