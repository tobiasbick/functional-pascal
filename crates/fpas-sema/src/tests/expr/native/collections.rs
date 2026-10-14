//! Canonical collection forms and static ordering contracts at the checker boundary.
//! See `docs/pascal/language/types/array/basics.md`.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};

#[test]
fn removed_native_aliases_have_canonical_replacements() {
    for (body, hint) in [
        ("discard 'abc'.CharAt(0);", "Text[Index]"),
        ("discard 'abc'.Contains('a');", "Substring in Text"),
        ("discard [1].Contains(1);", "Value in Items"),
        ("discard ['a': 1].ContainsKey('a');", "Key in Values"),
        ("discard 'a'.FromChar(2);", "RepeatStr"),
        (
            "var Text := 'a'; Text.FromCharInPlace(2);",
            "RepeatStrInPlace",
        ),
    ] {
        let errors = check_errors(&format!("program T; begin {body} end."));
        assert_eq!(errors.len(), 1, "{body}: {errors:?}");
        assert_eq!(errors[0].code, SEMA_UNKNOWN_NAME, "{errors:?}");
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains(hint)),
            "{errors:?}"
        );
    }
}

#[test]
fn every_index_search_rejects_integer_sentinel_consumers() {
    for expression in [
        "[1].IndexOf(1)",
        "[1].FindIndex(function(X: integer): boolean begin return X = 1; end function)",
        "'abc'.IndexOf('b')",
        "'abc'.LastIndexOf('b')",
    ] {
        let errors = check_errors(&format!(
            "program T; begin const Index: integer := {expression}; end."
        ));
        assert_eq!(errors.len(), 1, "{expression}: {errors:?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:?}");
        assert!(errors[0].message.contains("Option"), "{errors:?}");
        for comparison in [format!("{expression} >= 0"), format!("-1 = {expression}")] {
            let errors = check_errors(&format!("program T; begin discard {comparison}; end."));
            assert_eq!(errors.len(), 1, "{comparison}: {errors:?}");
            assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:?}");
            assert!(
                errors[0]
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains("Some(Index) or None")),
                "{errors:?}"
            );
        }
    }
}

#[test]
fn unsupported_sort_elements_fail_even_for_typed_empty_arrays() {
    for element in [
        "Point",
        "Choice",
        "Option of integer",
        "Result of (integer, string)",
        "array of integer",
        "dict of string to integer",
        "function(): integer",
        "procedure()",
        "task",
        "channel of integer",
    ] {
        for body in [
            format!("const Items: array of {element} := []; discard Items.Sort();"),
            format!("var Items: array of {element} := []; Items.SortInPlace();"),
            format!("const Items: array of {element} := [].Sort();"),
            format!("const Items: array of {element} := [].Sort().Reverse().Slice(0, 0);"),
        ] {
            let errors = check_errors(&format!(
                "program T; type Point = record X: integer; end record; type Choice = enum Number(Value: integer); Empty; end enum; begin {body} end."
            ));
            assert_eq!(errors.len(), 1, "{element}: {errors:?}");
            assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:?}");
            assert!(
                errors[0].message.contains("Sort") && errors[0].message.contains("Comparable"),
                "{errors:?}"
            );
        }
    }
    for (element, value) in [
        ("Point", "Point(X := 1)"),
        ("Choice", "Choice.Number(1)"),
        ("Option of integer", "Some(1)"),
        ("Result of (integer, string)", "Ok(1)"),
        ("array of integer", "[1]"),
        ("dict of string to integer", "['a': 1]"),
        (
            "function(): integer",
            "function(): integer begin return 1; end function",
        ),
        ("procedure()", "procedure() begin null; end procedure"),
    ] {
        let errors = check_errors(&format!(
            "program T; type Point = record X: integer; end record; type Choice = enum Number(Value: integer); Empty; end enum; begin const Items: array of {element} := [{value}]; discard Items.Sort(); end."
        ));
        assert_eq!(errors.len(), 1, "{element}: {errors:?}");
        assert!(errors[0].message.contains("Comparable"), "{errors:?}");
    }
}

#[test]
fn sort_checks_inferred_input_before_type_changing_or_terminal_operations() {
    for expression in [
        "[].Sort().Push(Point(X := 1))",
        "[].Sort().Concat([Point(X := 1)])",
        "[].Sort().Map(function(Value: Point): integer begin return Value.X; end function)",
        "[].Sort().IndexOf(Point(X := 1))",
        "[].Sort().FindIndex(function(Value: Point): boolean begin return Value.X = 1; end function)",
    ] {
        let errors = check_errors(&format!(
            "program T; type Point = record X: integer; end record; begin discard {expression}; end."
        ));
        assert_eq!(errors.len(), 1, "{expression}: {errors:?}");
        assert!(errors[0].message.contains("Comparable"), "{errors:?}");
    }
    for context in [
        "const Values: Option of array of Point := Some([].Sort());",
        "Consume([].Sort());",
    ] {
        let errors = check_errors(&format!(
            "program T; type Point = record X: integer; end record; procedure Consume(Values: array of Point); begin end procedure; begin {context} end."
        ));
        assert_eq!(errors.len(), 1, "{context}: {errors:?}");
        assert!(errors[0].message.contains("Comparable"), "{errors:?}");
    }
    check_ok(
        "program T; begin const Values: array of integer := [].Sort().Reverse().Slice(0, 0); const Pushed := [].Sort().Push(1); const Joined := [].Sort().Concat([1]); const Mapped := [].Sort().Map(function(Value: integer): string begin return 'x'; end function); const Index := [].Sort().IndexOf(1); end.",
    );
}

#[test]
fn generic_sorting_requires_an_ordering_guarantee() {
    for constraint in ["", ": Printable"] {
        for expression in ["Items.Sort()", "[].Sort().Reverse()"] {
            let errors = check_errors(&format!(
                "program T; function SortValues<T{constraint}>(Items: array of T): array of T; begin return {expression}; end function; begin end."
            ));
            assert_eq!(errors.len(), 1, "{errors:?}");
            assert!(
                errors[0]
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains("T: Comparable")),
                "{errors:?}"
            );
        }
    }
    for constraint in ["Comparable", "Numeric"] {
        check_ok(&format!(
            "program T; function Make<T: {constraint}>(): array of T; begin return [].Sort().Reverse(); end function; begin const Values: array of integer := Make(); end."
        ));
    }
    // Positive concrete and constrained generic cases execute in canonical_sorting_test.fpas.
    check_ok(
        "program T; function ReverseValues<T>(Items: array of T): array of T; begin return Items.Reverse(); end function; begin end.",
    );
}
