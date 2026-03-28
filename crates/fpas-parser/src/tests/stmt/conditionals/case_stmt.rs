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
