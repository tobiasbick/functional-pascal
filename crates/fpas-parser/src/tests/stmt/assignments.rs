use super::body_stmts;
use crate::ast::*;

#[test]
fn simple_assign() {
    let stmts = body_stmts("program T; begin X := 1 end.");
    match &stmts[0] {
        Stmt::Assign { target, value, .. } => {
            assert_eq!(target.parts.len(), 1);
            assert!(matches!(value, Expr::Integer(1, _)));
        }
        _ => panic!("expected Assign"),
    }
}

#[test]
fn field_assign() {
    let stmts = body_stmts("program T; begin P.X := 3.0 end.");
    match &stmts[0] {
        Stmt::Assign { target, .. } => {
            assert_eq!(target.parts.len(), 2);
        }
        _ => panic!("expected Assign"),
    }
}

#[test]
fn indexed_assign() {
    let stmts = body_stmts("program T; begin Arr[0] := 1 end.");
    match &stmts[0] {
        Stmt::Assign { target, .. } => {
            assert_eq!(target.parts.len(), 2);
            assert!(matches!(&target.parts[1], DesignatorPart::Index(_, _)));
        }
        _ => panic!("expected Assign"),
    }
}
