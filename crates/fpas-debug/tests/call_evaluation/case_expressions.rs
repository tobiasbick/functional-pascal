//! `case` expressions are rejected by debugger evaluation with an actionable hint.
//! See `docs/pascal/tools/debugger.md`.

use super::named_arguments::{evaluate, server};

#[test]
fn case_expressions_are_unsupported_in_debugger_evaluation() {
    let mut server = server("program DebugCase;\nbegin end.");
    let response = evaluate(
        &mut server,
        "case 1 of when 1: 'one'; else 'other'; end case",
    );
    assert_eq!(response["success"], false, "{response}");
    assert!(
        response.to_string().contains("`case` expressions"),
        "{response}"
    );
}
