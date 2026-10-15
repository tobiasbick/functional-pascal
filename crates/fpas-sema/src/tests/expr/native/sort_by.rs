//! Selector-shape and Comparable-key diagnostics at the language-analysis boundary.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

#[test]
fn sort_by_rejects_noncomparable_keys_even_on_empty_arrays() {
    for key in [
        "Point",
        "array of integer",
        "Option of integer",
        "dict of string to integer",
    ] {
        for receiver in [
            "const Items: array of integer := []; discard Items.SortBy(Key);",
            "var Items := [1]; Items.SortByInPlace(Key);",
        ] {
            let errors = check_errors(&format!(
                "program T; type Point = record X: integer; end record; function Key(X: integer): {key}; begin panic('unused'); end function; begin {receiver} end."
            ));
            assert_eq!(errors.len(), 1, "{key}: {errors:?}");
            assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:?}");
            assert!(errors[0].message.contains("Comparable keys"), "{errors:?}");
        }
    }
    for constraint in ["", ": Printable"] {
        let errors = check_errors(&format!(
            "program T; function Order<T, K{constraint}>(Items: array of T; F: function(X: T): K): array of T; begin return Items.SortBy(F); end function; begin end."
        ));
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].message.contains("Comparable keys"), "{errors:?}");
    }
    check_ok(
        "program T; function Order<T, K: Numeric>(Items: array of T; F: function(X: T): K): array of T; begin return Items.SortBy(F); end function; begin end.",
    );
}

#[test]
fn sort_by_rejects_invalid_selector_shapes() {
    for callback in [
        "1",
        "procedure(X: integer) begin null; end procedure",
        "function(): integer begin return 1; end function",
        "function(X: integer; Y: integer): integer begin return X; end function",
        "function(X: string): integer begin return X.Length(); end function",
        "function(var X: integer): integer begin return X; end function",
    ] {
        let errors = check_errors(&format!(
            "program T; begin discard [1].SortBy({callback}); end."
        ));
        assert!(!errors.is_empty(), "{callback} must be rejected");
        assert!(
            errors.iter().all(|error| error.code == SEMA_TYPE_MISMATCH),
            "{callback}: {errors:?}"
        );
    }
}
