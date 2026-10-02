use super::body_stmts;
use crate::ast::*;

#[test]
fn nested_block() {
    let stmts = body_stmts(r#"program T; begin begin X := 1; end; end program;"#);
    assert!(matches!(&stmts[0], Stmt::Block(_, _)));
}

#[test]
fn multiple_statements() {
    let stmts = body_stmts(r#"program T; begin X := 1; Y := 2; Z := 3; end program;"#);
    assert_eq!(stmts.len(), 3);
}

#[test]
fn no_semi_before_end() {
    let stmts = body_stmts(r#"program T; begin X := 1; end program;"#);
    assert_eq!(stmts.len(), 1);
}

#[test]
fn trailing_semicolon_before_end() {
    let stmts = body_stmts(r#"program T; begin X := 1; end program;"#);
    assert_eq!(stmts.len(), 1);
}
