//! Callback failures and explicit scalar invariants through compiled collection calls.
//! See `docs/pascal/language/types/array/higher-order.md`.

use super::run_program;
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

#[test]
fn index_search_propagates_predicate_failure_in_direct_and_nested_callbacks() {
    for expression in [
        "[1, 2].FindIndex(function(X: integer): boolean begin panic('predicate failed'); return false; end function)",
        "[[1, 2]].Map(function(Items: array of integer): Option of integer begin return Items.FindIndex(function(X: integer): boolean begin panic('predicate failed'); return false; end function); end function)",
    ] {
        let error = run_program(&format!("program T; begin discard {expression}; end."))
            .expect_err("predicate failures propagate");
        assert_eq!(error.code, RUNTIME_PROGRAM_PANIC, "{error:?}");
        assert!(error.message.contains("predicate failed"), "{error:?}");
    }
}

#[test]
fn explicit_single_scalar_repetition_contract_rejects_invalid_input() {
    for text in ["", "ab", "é"] {
        let error = run_program(&format!(
            "program T; function RepeatScalar(Text: string; Count: integer): string; requires Text.Length() = 1; begin return Text.RepeatStr(Count); end function; begin discard RepeatScalar('{text}', 2); end."
        )).expect_err("single-scalar user precondition");
        assert_eq!(error.code, RUNTIME_PROGRAM_PANIC, "{error:?}");
        assert!(
            error.message.contains("requires Text.Length() = 1"),
            "{error:?}"
        );
    }
}
