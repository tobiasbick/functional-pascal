//! Operator precedence, migration diagnostics, and structural regression coverage.
mod common;

#[test]
fn target_operator_expressions_keep_structure_comments_and_idempotence() {
    for expression in [
        "not X > 0 and Y > 0",
        "not not X in [1, 2]",
        "A and B and C",
        "A or (B and C)",
        "(A xor B) or C",
        "A - (B - C)",
        "(A - B) - C",
        "(not A) = B",
        "try GetValue() + 2 * 3",
        "A and // right operand\n not B = C",
    ] {
        common::assert_round_trip(
            expression,
            &format!("program T; begin X := {expression}; end program;"),
        );
    }
}
