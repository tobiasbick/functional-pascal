use super::parse_expr;
use crate::ast::*;
use crate::tests::parse_with_errors;

#[test]
fn invocation_suffixes_accept_expression_targets() {
    for source in [
        "Make()(1)",
        "Callbacks[0](1)",
        "(Callback)(1)",
        "[Callback][0](1)",
        "(function(X: integer): integer begin return X; end function)(1)",
    ] {
        let Expr::Postfix { operations, .. } = parse_expr(source) else {
            panic!("expected an expression-target call: {source}");
        };
        assert!(
            matches!(operations.last(), Some(PostfixOperation::Call { args, .. }) if args.len() == 1)
        );
    }
}

#[test]
fn malformed_invocation_recovers_with_structured_diagnostics() {
    for source in [
        "program P; begin discard Make()(1,); end program;",
        "program P; begin discard (F)(Name := 1); end program;",
        "program P; begin discard (F)(1; end program;",
        "program P; begin discard; end program;",
    ] {
        let (_, errors) = parse_with_errors(source);
        assert!(!errors.is_empty(), "{source}");
        assert!(
            errors
                .iter()
                .all(|error| error.as_diagnostic().span.is_some()),
            "{errors:#?}"
        );
    }
}

#[test]
fn call_result_followed_by_field() {
    match parse_expr("Foo().Bar") {
        Expr::Postfix {
            base, operations, ..
        } => {
            assert!(matches!(*base, Expr::Call { .. }));
            assert_eq!(operations.len(), 1);
            match &operations[0] {
                PostfixOperation::Field { name, .. } => assert_eq!(name, "Bar"),
                other => panic!("expected Field, got {other:?}"),
            }
        }
        other => panic!("expected Postfix, got {other:?}"),
    }
}

#[test]
fn call_result_followed_by_index() {
    match parse_expr("Foo()[0]") {
        Expr::Postfix {
            base, operations, ..
        } => {
            assert!(matches!(*base, Expr::Call { .. }));
            assert_eq!(operations.len(), 1);
            assert!(matches!(operations[0], PostfixOperation::Index { .. }));
        }
        other => panic!("expected Postfix, got {other:?}"),
    }
}

#[test]
fn call_result_followed_by_method_call() {
    match parse_expr("Foo().Transform(2)") {
        Expr::Postfix {
            base, operations, ..
        } => {
            assert!(matches!(*base, Expr::Call { .. }));
            assert_eq!(operations.len(), 1);
            match &operations[0] {
                PostfixOperation::MethodCall { name, args, .. } => {
                    assert_eq!(name, "Transform");
                    assert_eq!(args.len(), 1);
                }
                other => panic!("expected MethodCall, got {other:?}"),
            }
        }
        other => panic!("expected Postfix, got {other:?}"),
    }
}

#[test]
fn two_method_calls_followed_by_field() {
    match parse_expr("Foo().Transform(2).Scale(3).Value") {
        Expr::Postfix {
            base, operations, ..
        } => {
            assert!(matches!(*base, Expr::Call { .. }));
            assert_eq!(operations.len(), 3);
            assert!(matches!(
                &operations[0],
                PostfixOperation::MethodCall { name, .. } if name == "Transform"
            ));
            assert!(matches!(
                &operations[1],
                PostfixOperation::MethodCall { name, .. } if name == "Scale"
            ));
            assert!(matches!(
                &operations[2],
                PostfixOperation::Field { name, .. } if name == "Value"
            ));
        }
        other => panic!("expected Postfix, got {other:?}"),
    }
}

#[test]
fn parenthesized_call_result_followed_by_field() {
    match parse_expr("(Factory.Create()).Value") {
        Expr::Postfix {
            base, operations, ..
        } => {
            assert!(matches!(*base, Expr::Paren(_, _)));
            assert_eq!(operations.len(), 1);
            assert!(matches!(
                &operations[0],
                PostfixOperation::Field { name, .. } if name == "Value"
            ));
        }
        other => panic!("expected Postfix, got {other:?}"),
    }
}

#[test]
fn qualified_root_call_remains_call_inside_postfix_base() {
    match parse_expr("Std.Math.Sqrt(4.0).BitLength") {
        Expr::Postfix { base, .. } => match *base {
            Expr::Call {
                designator, args, ..
            } => {
                assert_eq!(designator.parts.len(), 3);
                assert_eq!(args.len(), 1);
            }
            other => panic!("expected Call base, got {other:?}"),
        },
        other => panic!("expected Postfix, got {other:?}"),
    }
}

#[test]
fn missing_identifier_after_dot_recovers() {
    let (_program, errors) = parse_with_errors("program T; begin return Foo(). end.");
    assert!(
        !errors.is_empty(),
        "expected parser diagnostic for missing identifier after `.`"
    );
}

#[test]
fn missing_rbracket_recovers() {
    let (_program, errors) = parse_with_errors("program T; begin return Foo()[0 end.");
    assert!(
        !errors.is_empty(),
        "expected parser diagnostic for missing `]`"
    );
}
