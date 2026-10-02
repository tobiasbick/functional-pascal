use super::body_stmts;
use crate::ast::*;

#[test]
fn inline_var() {
    let stmts = body_stmts(r#"program T; begin var X: integer := 42; end program;"#);
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::Var(var_def) => {
            assert_eq!(var_def.name, "X");
            assert!(matches!(var_def.value, Expr::Integer(42, _)));
        }
        _ => panic!("expected Var"),
    }
}

#[test]
fn inline_mutable_var() {
    let stmts = body_stmts(r#"program T; begin mutable var X: integer := 0; end program;"#);
    assert!(matches!(&stmts[0], Stmt::MutableVar(_)));
}
