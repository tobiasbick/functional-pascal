//! Contextual modifiers, unit segments, and ordinary As identifiers.

use fpas_language_service::SemanticTokenKind;

use super::fixture;

#[test]
fn contextual_as_and_alias_named_as_have_distinct_semantic_tokens() {
    let source = "program App;\nuses Demo.Math\n  // comment before modifier\n  aS // comment before alias\n  As;\nbegin\n  discard As.Answer(1);\nend.\n";
    let mut f = fixture(source);
    let tokens = f.service.semantic_tokens(&f.main).expect("tokens").value;
    let modifier = source.find("aS //").expect("modifier");
    let alias = source.find("  As;").expect("alias") + 2;
    let usage = source.find("As.Answer").expect("alias use");
    assert_eq!(
        tokens
            .iter()
            .filter(|t| t.kind == SemanticTokenKind::Keyword)
            .count(),
        1
    );
    for (offset, kind, declaration) in [
        (modifier, SemanticTokenKind::Keyword, false),
        (alias, SemanticTokenKind::Namespace, true),
        (usage, SemanticTokenKind::Namespace, false),
    ] {
        let token = tokens
            .iter()
            .find(|t| t.span.offset() == offset)
            .expect("semantic token");
        assert_eq!(token.kind, kind);
        assert_eq!(token.modifiers.declaration, declaration);
    }
}

#[test]
fn ordinary_as_declarations_are_never_keywords() {
    let source = "program App;\nconst As: integer := 1;\nbegin\n  discard As;\nend.\n";
    let mut f = fixture(source);
    let tokens = f.service.semantic_tokens(&f.main).expect("tokens").value;
    assert!(!tokens.iter().any(|t| t.kind == SemanticTokenKind::Keyword));
    for offset in [
        source.find("As:").expect("declaration"),
        source.find("As;").expect("reference"),
    ] {
        assert_eq!(
            tokens
                .iter()
                .find(|t| t.span.offset() == offset)
                .expect("ordinary identifier")
                .kind,
            SemanticTokenKind::Constant
        );
    }
}

#[test]
fn as_in_unit_paths_is_a_namespace_and_original_hidden_paths_are_unresolved() {
    let source = "program App;\nuses Demo.As as M;\nbegin\n  discard M.Answer(1);\n  discard Demo.As.Answer(1);\nend.\n";
    let mut f = fixture(source);
    std::fs::write(&f.unit, super::UNIT.replace("Demo.Math", "Demo.As"))
        .expect("unit segment named As");
    let tokens = f.service.semantic_tokens(&f.main).expect("tokens").value;
    let segment = source.find("Demo.As as").expect("import segment") + 5;
    assert_eq!(
        tokens
            .iter()
            .find(|t| t.span.offset() == segment)
            .expect("unit segment")
            .kind,
        SemanticTokenKind::Namespace
    );
    let hidden = source.rfind("Demo.As").expect("hidden source path");
    assert!(!tokens.iter().any(|t| t.span.offset() >= hidden));
}
