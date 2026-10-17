//! Pascal width specifiers in call arguments: one diagnostic per argument, recovery, and no false
//! positives.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use super::foreign_habit_assertions::assert_one_habit;
use crate::tests::parse_ok;
use fpas_diagnostics::codes::PARSE_FOREIGN_SYNTAX;

#[test]
fn width_specifiers_are_reported_once_per_argument() {
    assert_one_habit(
        "procedure Q(X: real);\nbegin\n  Show(X:0:2);\nend procedure;",
        PARSE_FOREIGN_SYNTAX,
        "Pascal width specifier",
        "X:0:2",
        "X",
    );
    assert_one_habit(
        "procedure Q(X: integer);\nbegin\n  Show('Total: ', X:8);\nend procedure;",
        PARSE_FOREIGN_SYNTAX,
        "Pascal width specifier",
        "X:8",
        "X",
    );
}

#[test]
fn named_arguments_are_not_width_specifiers() {
    parse_ok("program P;\n\nbegin\n  Show(Value := 1, Width := 2);\nend.");
}
