use super::super::body_stmts;
use crate::ast::*;

#[test]
fn if_then() {
    let stmts = body_stmts("program T; begin if X > 0 then Y := 1 end.");
    match &stmts[0] {
        Stmt::If {
            else_branch: None, ..
        } => {}
        _ => panic!("expected If without else"),
    }
}

#[test]
fn if_then_else() {
    let stmts = body_stmts("program T; begin if X > 0 then Y := 1 else Y := 2 end.");
    match &stmts[0] {
        Stmt::If {
            else_branch: Some(_),
            ..
        } => {}
        _ => panic!("expected If with else"),
    }
}

#[test]
fn else_if_chain() {
    let stmts = body_stmts(
        "program T; begin \
         if X = 1 then A := 1 \
         else if X = 2 then A := 2 \
         else A := 3 \
         end.",
    );
    match &stmts[0] {
        Stmt::If {
            else_branch: Some(else_stmt),
            ..
        } => {
            assert!(matches!(else_stmt.as_ref(), Stmt::If { .. }));
        }
        _ => panic!("expected else-if chain"),
    }
}

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

#[test]
fn nested_if_in_then_branch() {
    let stmts = body_stmts(
        "program T; begin \
         if A then \
           if B then X := 1 \
           else X := 2 \
         end.",
    );
    match &stmts[0] {
        Stmt::If {
            then_branch,
            else_branch: None,
            ..
        } => {
            assert!(matches!(
                then_branch.as_ref(),
                Stmt::If {
                    else_branch: Some(_),
                    ..
                }
            ));
        }
        _ => panic!("expected nested If"),
    }
}

#[test]
fn deeply_chained_else_if() {
    let stmts = body_stmts(
        "program T; begin \
         if X = 1 then A := 1 \
         else if X = 2 then A := 2 \
         else if X = 3 then A := 3 \
         else if X = 4 then A := 4 \
         else A := 0 \
         end.",
    );

    let mut current = &stmts[0];
    for _ in 0..3 {
        match current {
            Stmt::If {
                else_branch: Some(else_stmt),
                ..
            } => current = else_stmt.as_ref(),
            _ => panic!("expected If in chain"),
        }
    }

    match current {
        Stmt::If {
            else_branch: Some(else_stmt),
            ..
        } => {
            assert!(!matches!(else_stmt.as_ref(), Stmt::If { .. }));
        }
        _ => panic!("expected final If with plain else"),
    }
}
