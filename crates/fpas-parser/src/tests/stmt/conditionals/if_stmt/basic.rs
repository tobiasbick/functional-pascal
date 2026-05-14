use super::*;

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