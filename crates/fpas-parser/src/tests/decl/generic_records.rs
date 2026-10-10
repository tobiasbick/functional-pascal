//! Generic record declarations and annotation-only type applications.

use super::*;

#[test]
fn record_parameters_constraints_and_nested_applications_parse() {
    let program = parse_ok(
        "program T; type Box of T = record Value: T; end record; type Pair of (K: Comparable, V) = record Key: K; Value: V; end record; begin const P: Pair of (string, Box of array of integer) := Pair(Key := 'a', Value := Box(Value := [1])); end.",
    );
    let Decl::TypeDef(pair) = &program.declarations[1] else {
        panic!("record declaration")
    };
    assert_eq!(pair.type_params.len(), 2);
    assert_eq!(
        pair.type_params[0].constraint.as_deref(),
        Some("Comparable")
    );
    let Stmt::Const(value) = &program.body[0] else {
        panic!("typed binding")
    };
    let TypeExpr::Application { arguments, .. } = &value.type_expr else {
        panic!("record application")
    };
    assert_eq!(arguments.len(), 2);
    assert!(matches!(&arguments[1], TypeExpr::Application { .. }));
}

#[test]
fn rejected_record_forms_explain_canonical_spelling_and_recover() {
    for (source, hint) in [
        (
            "type Box<T> = record Value: T; end record;",
            "type Box of T",
        ),
        (
            "type Pair of K, V = record Key: K; Value: V; end record;",
            "type Pair of (K, V)",
        ),
        (
            "type Box of (T) = record Value: T; end record;",
            "type Box of T",
        ),
        ("type Alias of T = integer;", "generic record or enum"),
    ] {
        let (_, errors) = parse_with_errors(&format!("program T; {source} begin end."));
        assert!(
            errors.iter().any(|error| error
                .as_diagnostic()
                .help
                .as_deref()
                .is_some_and(|value| value.contains(hint))),
            "{errors:?}"
        );
    }
}

#[test]
fn explicit_constructor_type_arguments_have_an_annotation_hint() {
    let (_, errors) = parse_with_errors(
        "program T; begin const P: Pair of (integer, string) := Pair of (integer, string)(Key := 1, Value := 'a'); end.",
    );
    assert!(
        errors.iter().any(|error| error
            .as_diagnostic()
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("annotation"))),
        "{errors:?}"
    );
}
