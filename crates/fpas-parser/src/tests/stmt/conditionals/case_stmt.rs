use super::super::body_stmts;
use crate::ast::*;
use crate::tests::parse_with_errors;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;

#[test]
fn nested_pattern_nodes_preserve_explicit_bindings_and_payload_wildcards() {
    let statements = body_stmts(
        "program Main; begin case Item of
        when Choice.Present(Option.Some(const Value)), Choice.Present(Option.Some(_)): null;
        when Choice.Missing: null; end case; end program;",
    );
    let Stmt::Case { arms, .. } = &statements[0] else {
        panic!("case statement");
    };
    let Pattern::Variant { arguments, .. } = &arms[0].labels[0] else {
        panic!("recursive pattern");
    };
    let Pattern::Variant { arguments, .. } = &arguments[0] else {
        panic!("nested option");
    };
    assert!(matches!(&arguments[0], Pattern::Binding { name, .. } if name == "Value"));
}

#[test]
fn case_basic() {
    let stmts = body_stmts(
        r#"program T; begin case X of when 1: A := 1; when 2: A := 2; end case; end program;"#,
    );
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
    let stmts =
        body_stmts(r#"program T; begin case X of when 0..9: A := 1; end case; end program;"#);
    match &stmts[0] {
        Stmt::Case { arms, .. } => match &arms[0].labels[0] {
            Pattern::Value { end, .. } => assert!(end.is_some()),
            _ => panic!("expected Value label"),
        },
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_with_else() {
    let stmts = body_stmts(
        r#"program T; begin case X of when 1: A := 1; else A := 0; end case; end program;"#,
    );
    match &stmts[0] {
        Stmt::Case { else_body, .. } => {
            assert!(else_body.is_some());
        }
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_with_only_else_is_rejected_and_keeps_else_body_for_recovery() {
    let (program, errors) = parse_with_errors("program T; begin case X of else A := 0 end end.");
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
    let stmts =
        body_stmts(r#"program T; begin case X of when 1, 2, 3: A := 1; end case; end program;"#);
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
        r#"program T; begin case S of when Shape.Circle(const R) if R > 10.0: A := 1; end case; end program;"#,
    );
    match &stmts[0] {
        Stmt::Case { arms, .. } => {
            assert!(arms[0].guard.is_some());
            let Pattern::Variant { arguments, .. } = &arms[0].labels[0] else {
                panic!("expected enum variant pattern");
            };
            assert_eq!(arguments.len(), 1);
            assert!(matches!(&arguments[0], Pattern::Binding { name, .. } if name == "R"));
        }
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_with_destructure_pattern() {
    let stmts = body_stmts(
        r#"program T; begin case R of when Result.Ok(const V): A := 1; when Result.Error(const E): A := 2; end case; end program;"#,
    );
    match &stmts[0] {
        Stmt::Case { arms, .. } => match &arms[0].labels[0] {
            Pattern::Variant {
                designator,
                arguments,
                ..
            } => {
                assert!(
                    matches!(&designator.parts[1], DesignatorPart::Ident(name, _) if name == "Ok")
                );
                assert!(matches!(&arguments[0], Pattern::Binding { name, .. } if name == "V"));
            }
            _ => panic!("expected destructure label"),
        },
        _ => panic!("expected Case"),
    }
}
