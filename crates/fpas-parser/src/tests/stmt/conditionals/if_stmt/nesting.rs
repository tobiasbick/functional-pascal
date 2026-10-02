use super::*;

#[test]
fn nested_if_in_then_branch() {
    let stmts = body_stmts(
        r#"program T; begin if A then if B then X := 1; else X := 2; end if; end if; end program;"#,
    );
    match &stmts[0] {
        Stmt::If {
            then_branch,
            else_branch: None,
            ..
        } => {
            assert!(matches!(
                first(then_branch),
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
        r#"program T; begin if X = 1 then A := 1; else if X = 2 then A := 2; else if X = 3 then A := 3; else if X = 4 then A := 4; else A := 0; end if; end if; end if; end if; end program;"#,
    );

    let mut current = &stmts[0];
    for _ in 0..3 {
        match current {
            Stmt::If {
                else_branch: Some(else_stmt),
                ..
            } => current = first(else_stmt),
            _ => panic!("expected If in chain"),
        }
    }

    match current {
        Stmt::If {
            else_branch: Some(else_stmt),
            ..
        } => {
            assert!(!matches!(first(else_stmt), Stmt::If { .. }));
        }
        _ => panic!("expected final If with plain else"),
    }
}
