use super::super::body_stmts;
use crate::ast::*;

#[test]
fn case_basic() {
    let stmts = body_stmts("program T; begin case X of 1: A := 1; 2: A := 2 end end.");
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
    let stmts = body_stmts("program T; begin case X of 0..9: A := 1 end end.");
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
    let stmts = body_stmts("program T; begin case X of 1: A := 1 else A := 0 end end.");
    match &stmts[0] {
        Stmt::Case { else_body, .. } => {
            assert!(else_body.is_some());
        }
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_multiple_labels() {
    let stmts = body_stmts("program T; begin case X of 1, 2, 3: A := 1 end end.");
    match &stmts[0] {
        Stmt::Case { arms, .. } => {
            assert_eq!(arms[0].labels.len(), 3);
        }
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_with_guard_and_enum_pattern() {
    let stmts =
        body_stmts("program T; begin case S of Shape.Circle(R) if R > 10.0: A := 1 end end.");
    match &stmts[0] {
        Stmt::Case { arms, .. } => {
            assert!(arms[0].guard.is_some());
            let CaseLabel::Value {
                start, end: None, ..
            } = &arms[0].labels[0]
            else {
                panic!("expected enum-pattern label");
            };
            let Expr::Call { args, .. } = start else {
                panic!("expected enum-pattern call");
            };
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0], Expr::Designator(_)));
        }
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_with_destructure_pattern() {
    let stmts = body_stmts("program T; begin case R of Ok(V): A := 1; Error(E): A := 2 end end.");
    match &stmts[0] {
        Stmt::Case { arms, .. } => match &arms[0].labels[0] {
            CaseLabel::Destructure {
                variant, binding, ..
            } => {
                assert_eq!(*variant, DestructureVariant::Ok);
                assert_eq!(binding.as_deref(), Some("V"));
            }
            _ => panic!("expected destructure label"),
        },
        _ => panic!("expected Case"),
    }
}

#[test]
fn case_with_nested_pattern_wildcard() {
    let stmts = body_stmts("program T; begin case E of Expr.Mul(Expr.Num(0), _): A := 1 end end.");
    match &stmts[0] {
        Stmt::Case { arms, .. } => {
            let CaseLabel::Value {
                start, end: None, ..
            } = &arms[0].labels[0]
            else {
                panic!("expected nested-pattern label");
            };
            let Expr::Call { args, .. } = start else {
                panic!("expected outer call");
            };
            let Expr::Designator(wildcard) = &args[1] else {
                panic!("expected wildcard designator");
            };
            match &wildcard.parts[0] {
                DesignatorPart::Ident(name, _) => assert_eq!(name, "_"),
                _ => panic!("expected wildcard identifier"),
            }
        }
        _ => panic!("expected Case"),
    }
}
