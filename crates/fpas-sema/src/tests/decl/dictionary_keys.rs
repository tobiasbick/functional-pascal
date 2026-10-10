//! Dictionary key types must support equality.
//!
//! Documentation: `docs/pascal/language/types/dictionaries.md`

use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

use super::{check_errors, check_ok};

const TYPES: &str = "program T;
  type Color = enum Red; Green; end enum;
  type UserId = distinct integer;
  type Point = record X: integer; Y: integer; end record;
  type Bag = record Items: array of integer; end record;
  function One(): integer; begin return 1; end function;";

#[test]
fn keys_whose_values_compare_are_accepted() {
    check_ok(&format!(
        "{TYPES}
         function Index<K: Comparable>(Key: K): dict of K to integer;
         begin return [Key: 1]; end function;
         type Later = record Name: string; end record;
         begin
           const Text: dict of string to integer := ['a': 1];
           const Numbers: dict of real to boolean := [1.5: true];
           const Colors: dict of Color to string := [Color.Red: 'red'];
           const Ids: dict of UserId to string := [UserId(1): 'root'];
           const Points: dict of Point to string := [Point(X := 1, Y := 2): 'p'];
           const Optional: dict of option of integer to string := [Some(1): 'one'];
           const Declared: dict of Later to integer := [Later(Name := 'x'): 1];
         end."
    ));
}

#[test]
fn keys_without_equality_are_rejected_once_per_written_type() {
    for (key, value) in [
        ("array of integer", "[1]"),
        ("function(): integer", "One"),
        ("Bag", "Bag(Items := [1])"),
        ("Option of array of integer", "Some([1])"),
        ("dict of string to integer", "['a': 1]"),
    ] {
        let errors = check_errors(&format!(
            "{TYPES}
             begin const D: dict of {key} to string := [{value}: 'x']; end."
        ));
        assert_eq!(errors.len(), 1, "{key}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{key}: {errors:#?}");
        assert!(
            errors[0].message.contains(&format!(
                "Dictionary key type `{key}` does not support equality"
            )),
            "{key}: {errors:#?}"
        );
    }
}

#[test]
fn unconstrained_generic_keys_ask_for_comparable() {
    let errors = check_errors(
        "program T;
         function Index<K>(Key: K): dict of K to integer;
         begin return [Key: 1]; end function;
         begin end.",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0]
            .help
            .as_deref()
            .is_some_and(|help| help.contains("`: Comparable`")),
        "{errors:#?}"
    );
}

#[test]
fn inferred_literal_keys_are_checked_without_a_written_type() {
    let errors = check_errors(&format!(
        "{TYPES}
         begin const Size: integer := [[1]: 'x'].Length(); end."
    ));
    assert!(
        errors.iter().any(|error| error
            .message
            .contains("Dictionary key type `array of integer` does not support equality")),
        "{errors:#?}"
    );
}
