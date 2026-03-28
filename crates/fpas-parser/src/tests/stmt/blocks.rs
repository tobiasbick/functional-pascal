use super::body_stmts;
use crate::ast::*;

#[test]
fn nested_block() {
    let stmts = body_stmts("program T; begin begin X := 1 end end.");
    assert!(matches!(&stmts[0], Stmt::Block(_, _)));
}

#[test]
fn multiple_statements() {
    let stmts = body_stmts("program T; begin X := 1; Y := 2; Z := 3 end.");
    assert_eq!(stmts.len(), 3);
}

#[test]
fn no_semi_before_end() {
    let stmts = body_stmts("program T; begin X := 1 end.");
    assert_eq!(stmts.len(), 1);
}
