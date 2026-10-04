//! Equatable keys, forward nominal declarations, and rejected identity components.

use super::super::{check_errors, check_ok};

#[test]
fn dictionary_types_reject_unsupported_keys_transitively() {
    for key in [
        "function(): integer",
        "task of (integer)",
        "channel of (integer)",
        "Option of (function(): integer)",
        "array of (function(): integer)",
        "dict of (string, function(): integer)",
    ] {
        let errors = check_errors(&format!(
            r#"program Main; const Value: dict of ({key}, integer) := [:]; begin null; end program;"#
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Unsupported dictionary key type")),
            "{key}: {errors:#?}"
        );
    }
}

#[test]
fn dictionary_headers_check_forward_record_key_components_after_resolution() {
    let errors = check_errors(
        "program Main;
        type Lookup = dict of (Key, integer);
        type Key = record Action: function(): integer; end record;
        begin null; end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Unsupported dictionary key type")),
        "{errors:#?}"
    );
    check_ok(
        r#"program Main;
        type Lookup = dict of (Key, integer);
        type Key = record Value: integer; end record;
        const Values: Lookup := [:]; begin null; end program;"#,
    );
}

#[test]
fn generic_dictionary_keys_require_and_forward_equality_capabilities() {
    for constraint in ["Equatable", "Comparable", "Numeric"] {
        check_ok(&format!(
            r#"program Main;
            type Table of (K: {constraint}, V) = record Values: dict of (K, V); end record;
            function Empty of (K: {constraint}, V)(): dict of (K, V); begin return [:]; end function;
            begin const Value: Table of (integer, string) := Table(Values := [:]); end program;"#
        ));
    }
    let errors = check_errors(
        "program Main; type Table of (K, V) = record Values: dict of (K, V); end record; begin null; end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Unsupported dictionary key type")),
        "{errors:#?}"
    );
}

#[test]
fn dictionary_literal_keys_reject_nested_callables_without_restricting_values() {
    for key in ["Action", "[Action]", "Option.Some(Action)"] {
        let errors = check_errors(&format!(
            "program Main;
            function Action(): integer; begin return 42; end function;
            function Identity of (T)(Value: T): T; begin return Value; end function;
            begin discard Identity([{key}: 1]); end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Unsupported dictionary key type")),
            "{key}: {errors:#?}"
        );
    }
    check_ok(
        r#"program Main; function Action(): integer; begin return 42; end function;
        begin const Values: dict of (integer, function(): integer) := [1: Action]; end program;"#,
    );
}

#[test]
fn recursive_equatable_data_and_mapping_keys_use_supported_value_representations() {
    check_ok(
        r#"program Main;
        type Tree = enum Leaf(Value: integer); Branch(Values: array of (Tree)); end enum;
        const Trees: dict of (Tree, integer) := [Tree.Leaf(1): 42];
        const Mappings: dict of (dict of (string, integer), integer) := [['one': 1]: 42];
        begin null; end program;"#,
    );
}
