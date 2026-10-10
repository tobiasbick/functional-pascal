//! Generic enum headers, payload applications, and constructor diagnostics.

use super::*;

#[test]
fn generic_enum_parameters_and_recursive_payloads_parse() {
    let program = parse_ok(
        "program T; type Tree of (K: Comparable, V) = enum Leaf(Key: K; Value: V); Node(Left: Tree of (K, V); Right: Tree of (K, V)); Empty; end enum; begin end.",
    );
    let Decl::TypeDef(definition) = &program.declarations[0] else {
        panic!("enum declaration")
    };
    assert_eq!(definition.type_params.len(), 2);
    assert_eq!(
        definition.type_params[0].constraint.as_deref(),
        Some("Comparable")
    );
    let TypeBody::Enum(enumeration) = &definition.body else {
        panic!("enum body")
    };
    assert!(
        matches!(&enumeration.members[1].fields[0].type_expr, TypeExpr::Application { arguments, .. } if arguments.len() == 2)
    );
}

#[test]
fn explicit_enum_constructor_applications_require_annotations() {
    for value in [
        "Lookup of string.Found('x')",
        "Lookup of string.Missing",
        "Choice of (integer, string).Left(1)",
    ] {
        let (_, errors) =
            parse_with_errors(&format!("program T; begin const Value := {value}; end."));
        assert!(
            errors.iter().any(|error| error
                .as_diagnostic()
                .help
                .as_deref()
                .is_some_and(|hint| hint.contains("annotation"))),
            "{errors:?}"
        );
    }
}
