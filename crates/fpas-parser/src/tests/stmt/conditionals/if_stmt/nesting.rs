use super::*;

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