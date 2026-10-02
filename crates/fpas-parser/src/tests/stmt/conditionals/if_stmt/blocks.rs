use super::*;

#[test]
fn if_then_with_block() {
    let stmts = body_stmts(
        r#"program T; begin if X > 10 then begin Y := 1; Z := 2; end; end if; end program;"#,
    );
    match &stmts[0] {
        Stmt::If {
            then_branch,
            else_branch: None,
            ..
        } => {
            assert!(matches!(first(then_branch), Stmt::Block(..)));
        }
        _ => panic!("expected If with block then-branch"),
    }
}

#[test]
fn if_then_else_with_blocks() {
    let stmts = body_stmts(
        r#"program T; begin if X > 10 then begin Y := 1; end; else begin Y := 2; end; end if; end program;"#,
    );
    match &stmts[0] {
        Stmt::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        } => {
            assert!(matches!(first(then_branch), Stmt::Block(..)));
            assert!(matches!(first(else_branch), Stmt::Block(..)));
        }
        _ => panic!("expected If with block branches"),
    }
}
