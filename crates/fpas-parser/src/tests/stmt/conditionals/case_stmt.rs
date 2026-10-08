use super::super::body_stmts;
use crate::ast::*;
use crate::tests::parse_with_errors;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;

#[test]
fn case_basic() {
    let stmts =
        body_stmts("program T; begin case X of when 1: A := 1; when 2: A := 2; end case; end.");
    match &stmts[0] {
        Stmt::Case {
            arms, else_body, ..
        } => {
            assert_eq!(arms.len(), 2);
            assert!(else_body.is_none());
        }
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_with_range() {
    let stmts = body_stmts("program T; begin case X of when 0..9: A := 1; end case; end.");
    match &stmts[0] {
        Stmt::Case { arms, .. } => match &arms[0].labels[0] {
            CaseLabel::Value { end, .. } => assert!(end.is_some()),
            _ => panic!("expected Value label"),
        },
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_with_else() {
    let stmts =
        body_stmts("program T; begin case X of when 1: A := 1; else A := 0; end case; end.");
    match &stmts[0] {
        Stmt::Case { else_body, .. } => {
            assert!(else_body.is_some());
        }
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_with_only_else_is_rejected_and_keeps_else_body_for_recovery() {
    let (program, errors) =
        parse_with_errors("program T; begin case X of else A := 0; end case; end.");
    assert!(
        errors.iter().any(|error| error
            .as_parser_error()
            .is_some_and(|error| error.code == PARSE_EXPECTED_TOKEN
                && error.message == "Expected at least one case arm")),
        "{errors:#?}"
    );
    match &program.body[0] {
        Stmt::Case {
            arms, else_body, ..
        } => {
            assert!(arms.is_empty());
            assert!(else_body.is_some());
        }
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_multiple_labels() {
    let stmts = body_stmts("program T; begin case X of when 1, 2, 3: A := 1; end case; end.");
    match &stmts[0] {
        Stmt::Case { arms, .. } => {
            assert_eq!(arms[0].labels.len(), 3);
        }
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_with_guard_and_enum_pattern() {
    let stmts = body_stmts(
        "program T; begin case S of when Shape.Circle(const R) if R > 10.0: A := 1; end case; end.",
    );
    match &stmts[0] {
        Stmt::Case { arms, .. } => {
            assert!(arms[0].guard.is_some());
            let CaseLabel::Pattern(Pattern::Variant { fields, .. }) = &arms[0].labels[0] else {
                panic!("expected enum-pattern label");
            };
            assert_eq!(fields.len(), 1);
            assert!(matches!(&fields[0].pattern, Pattern::Binding { name, .. } if name == "R"));
        }
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_with_destructure_pattern() {
    let stmts = body_stmts(
        "program T; begin case R of when Ok(const V): A := 1; when Error(const E): A := 2; end case; end.",
    );
    match &stmts[0] {
        Stmt::Case { arms, .. } => match &arms[0].labels[0] {
            CaseLabel::Pattern(Pattern::Destructure {
                variant, payload, ..
            }) => {
                assert_eq!(*variant, DestructureVariant::Ok);
                assert!(
                    matches!(payload.as_deref(), Some(Pattern::Binding { name, .. }) if name == "V")
                );
            }
            _ => panic!("expected destructure label"),
        },
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_patterns_parse_bindings_wildcards_and_scalar_bindings() {
    let stmts = body_stmts(
        "program T; begin case S of when Shape.Rect(const W, _): A := 1; when Shape.Circle(Radius := const R): A := 2; end case; case N of when const V if V > 0: A := 3; end case; case O of when Error(_): A := 4; end case; end.",
    );
    let Stmt::Case { arms, .. } = &stmts[0] else {
        panic!("expected Case");
    };
    let CaseLabel::Pattern(Pattern::Variant { fields, .. }) = &arms[0].labels[0] else {
        panic!("expected variant pattern");
    };
    assert!(matches!(&fields[0].pattern, Pattern::Binding { name, .. } if name == "W"));
    assert!(matches!(fields[1].pattern, Pattern::Wildcard(_)));
    let CaseLabel::Pattern(Pattern::Variant { fields, .. }) = &arms[1].labels[0] else {
        panic!("expected named variant pattern");
    };
    assert_eq!(
        fields[0].label.as_ref().map(|(name, _)| name.as_str()),
        Some("Radius")
    );

    let Stmt::Case { arms, .. } = &stmts[1] else {
        panic!("expected scalar Case");
    };
    assert!(matches!(&arms[0].labels[0], CaseLabel::Binding { name, .. } if name == "V"));

    let Stmt::Case { arms, .. } = &stmts[2] else {
        panic!("expected Result Case");
    };
    assert!(matches!(
        &arms[0].labels[0],
        CaseLabel::Pattern(Pattern::Destructure {
            variant: DestructureVariant::Error,
            payload: Some(payload),
            ..
        }) if matches!(**payload, Pattern::Wildcard(_))
    ));
}

#[test]
fn plain_payload_identifier_is_kept_for_the_semantic_diagnostic() {
    let stmts = body_stmts("program T; begin case O of when Some(Value): A := 1; end case; end.");
    let Stmt::Case { arms, .. } = &stmts[0] else {
        panic!("expected Case");
    };
    assert!(matches!(
        &arms[0].labels[0],
        CaseLabel::Pattern(Pattern::Destructure {
            payload: Some(payload),
            ..
        }) if matches!(**payload, Pattern::Value(Expr::Designator(_)))
    ));
}

#[test]
fn nested_patterns_parse_recursively() {
    let stmts = body_stmts(
        "program T; begin case R of when Ok(Some(Shape.Rect(const W, 0))): A := 1; end case; end.",
    );
    let Stmt::Case { arms, .. } = &stmts[0] else {
        panic!("expected Case");
    };
    let CaseLabel::Pattern(Pattern::Destructure {
        variant: DestructureVariant::Ok,
        payload: Some(some),
        ..
    }) = &arms[0].labels[0]
    else {
        panic!("expected Ok pattern");
    };
    let Pattern::Destructure {
        variant: DestructureVariant::Some,
        payload: Some(inner),
        ..
    } = &**some
    else {
        panic!("expected nested Some pattern");
    };
    let Pattern::Variant { fields, .. } = &**inner else {
        panic!("expected nested variant pattern");
    };
    assert!(matches!(&fields[0].pattern, Pattern::Binding { name, .. } if name == "W"));
    assert!(matches!(
        &fields[1].pattern,
        Pattern::Value(Expr::Integer(0, _))
    ));
}
