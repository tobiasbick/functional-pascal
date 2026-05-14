use super::*;

#[test]
fn if_then_with_block() {
    let stmts = body_stmts(
        "program T; begin \
         if X > 10 then \
         begin \
           Y := 1; \
           Z := 2 \
         end \
         end.",
    );
    match &stmts[0] {
        Stmt::If {
            then_branch,
            else_branch: None,
            ..
        } => {
            assert!(matches!(then_branch.as_ref(), Stmt::Block(..)));
        }
        _ => panic!("expected If with block then-branch"),
    }
}

#[test]
fn if_then_else_with_blocks() {
    let stmts = body_stmts(
        "program T; begin \
         if X > 10 then \
         begin \
           Y := 1 \
         end \
         else \
         begin \
           Y := 2 \
         end \
         end.",
    );
    match &stmts[0] {
        Stmt::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        } => {
            assert!(matches!(then_branch.as_ref(), Stmt::Block(..)));
            assert!(matches!(else_branch.as_ref(), Stmt::Block(..)));
        }
        _ => panic!("expected If with block branches"),
    }
}