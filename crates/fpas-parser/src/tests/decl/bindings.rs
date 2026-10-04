use super::*;

#[test]
fn mutable_is_an_ordinary_identifier_after_keyword_removal() {
    parse_ok(
        "program T; function Identity(Mutable: integer): integer; begin return Mutable; end function; begin var Mutable := Identity(1); Mutable := 2; end program;",
    );
}

#[test]
fn local_bindings_share_optional_initializer_annotations() {
    let program = parse_ok(
        "program T; begin const A := Compute(); var B := [1, 2]; const C: real := 1; var D: integer := 2; end program;",
    );
    for (index, inferred) in [(0, true), (1, true), (2, false), (3, false)] {
        let definition = match &program.body[index] {
            Stmt::Const(definition) | Stmt::Var(definition) => definition,
            other => panic!("expected binding, got {other:?}"),
        };
        assert_eq!(definition.type_expr.is_none(), inferred);
    }
    assert!(matches!(program.body[0], Stmt::Const(_)));
    assert!(matches!(program.body[1], Stmt::Var(_)));
}

#[test]
fn top_level_bindings_require_annotations() {
    for source in [
        "program T; const A := 1; begin null; end program;",
        "program T; var A := 1; begin null; end program;",
        "unit U; public const A := 1; end unit;",
    ] {
        let (_, errors) = parse_compilation_unit_with_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.as_diagnostic().message.contains("explicit type")),
            "{errors:#?}"
        );
    }
}

#[test]
fn obsolete_mutable_binding_syntax_has_a_migration_hint() {
    for source in [
        "program T; mutable var A: integer := 1; begin null; end program;",
        "program T; begin mutable var A := 1; end program;",
    ] {
        let (_, errors) = parse_with_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.as_diagnostic().message.contains("obsolete")
                    && error
                        .as_diagnostic()
                        .help
                        .as_deref()
                        .is_some_and(|help| help.contains("var Value"))),
            "{errors:#?}"
        );
    }
}
